use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error};
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio::{select, task};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use uuid::Uuid;

use crate::mqtt::{MqttPublishEvent, MqttService, QoS};
use crate::publish::TriggerError;

#[derive(Clone, Debug)]
pub enum Command {
    NoMoreTasksPending,
}

struct JobContext {
    count: Option<u32>,
}

impl JobContext {
    pub fn new() -> Self {
        Self { count: None }
    }
}

struct JobContextStorage {
    contexts: HashMap<Uuid, JobContext>,
}

impl JobContextStorage {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    pub fn get_or_create_context(&mut self, uuid: &Uuid) -> &mut JobContext {
        if !self.contexts.contains_key(uuid) {
            let context = JobContext::new();
            self.contexts.insert(*uuid, context);
        };

        self.contexts.get_mut(uuid).unwrap()
    }

    pub fn exists(&self, uuid: &Uuid) -> bool {
        self.contexts.contains_key(uuid)
    }

    pub fn remove(&mut self, uuid: &Uuid) -> bool {
        self.contexts.remove(uuid).is_some()
    }
}

pub struct TriggerPeriodic {
    scheduler: Arc<Mutex<JobScheduler>>,
    mqtt_service: Arc<Mutex<dyn MqttService>>,
    sender_data: broadcast::Sender<(String, QoS, bool, Vec<u8>)>,
    job_contexts: Arc<Mutex<JobContextStorage>>,
    sender_command: broadcast::Sender<Command>,
}

impl TriggerPeriodic {
    pub async fn new(mqtt_service: Arc<Mutex<dyn MqttService>>) -> Self {
        let (sender_data, _) = broadcast::channel::<(String, QoS, bool, Vec<u8>)>(32);
        let (sender_command, _) = broadcast::channel::<Command>(4);

        Self {
            scheduler: Arc::new(Mutex::new(
                JobScheduler::new()
                    .await
                    .expect("Could not start job scheduler"),
            )),
            mqtt_service,
            sender_data,
            job_contexts: Arc::new(Mutex::new(JobContextStorage::new())),
            sender_command,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_schedule(
        &mut self,
        interval: &Duration,
        count: &Option<u32>,
        initial_delay: &Duration,
        topic: &str,
        qos: &QoS,
        retain: bool,
        payload: Vec<u8>,
    ) -> Result<(), TriggerError> {
        let qos = *qos;

        let scheduler = self.scheduler.clone();
        let initial_delay = *initial_delay;
        let contexts = self.job_contexts.clone();
        let count = *count;
        let interval = *interval;
        let topic = topic.to_owned();

        match count {
            Some(count) => {
                if count > 0 {
                    let job_initial = Self::create_job_one_shot(
                        &initial_delay,
                        retain,
                        qos,
                        &payload,
                        topic.as_ref(),
                        self.sender_data.clone(),
                    )?;

                    scheduler.lock().await.add(job_initial).await?;

                    if count > 1 {
                        let sender_data = self.sender_data.clone();

                        task::spawn(async move {
                            tokio::time::sleep(initial_delay).await;

                            let Ok(job_repeated) = Self::create_job_repeated_count(
                                contexts,
                                &interval,
                                retain,
                                qos,
                                &payload,
                                topic.as_ref(),
                                sender_data,
                                count - 1,
                            ) else {
                                error!("Error while scheduling repeated job");
                                return;
                            };

                            if let Err(e) = scheduler.lock().await.add(job_repeated).await {
                                error!("Error while adding repeated job to scheduler: {e:?}")
                            };
                        });
                    }
                } else {
                    debug!("Not adding task to publish to topic {topic}, count is zero");
                }
            }
            None => {
                let job_initial = Self::create_job_one_shot(
                    &initial_delay,
                    retain,
                    qos,
                    &payload,
                    topic.as_ref(),
                    self.sender_data.clone(),
                )?;

                scheduler.lock().await.add(job_initial).await?;

                let sender_data = self.sender_data.clone();

                task::spawn(async move {
                    tokio::time::sleep(initial_delay).await;

                    let Ok(job_repeated) = Self::create_job_repeated_forever(
                        &interval,
                        retain,
                        qos,
                        payload,
                        topic.as_ref(),
                        sender_data,
                    ) else {
                        error!("Error while scheduling repeated job");
                        return;
                    };

                    if let Err(e) = scheduler.lock().await.add(job_repeated).await {
                        error!("Error while adding repeated job to scheduler: {e:?}")
                    };
                });
            }
        };

        Ok(())
    }

    pub fn get_receiver_command(&self) -> broadcast::Receiver<Command> {
        self.sender_command.subscribe()
    }

    pub async fn start(
        &self,
        receiver_exit: BroadcastReceiver<()>,
    ) -> Result<JoinHandle<()>, TriggerError> {
        let mut receiver = self.sender_data.subscribe();
        let mut receiver_exit = receiver_exit;
        let mqtt_service = self.mqtt_service.clone();
        let scheduler = self.scheduler.clone();
        let sender_command = self.sender_command.clone();

        async fn is_task_pending(scheduler: &Arc<Mutex<JobScheduler>>, sender_command: &broadcast::Sender<Command>) -> bool {
            match scheduler.lock().await.time_till_next_job().await {
                Ok(value) => {
                    if value.is_none() {
                        debug!("No more pending tasks, exiting scheduler");
                        let _ = sender_command.send(Command::NoMoreTasksPending);
                    }

                    !value.is_none()
                }
                Err(_) => false,
            }
        }

        let task_handle = task::spawn(async move {
            debug!("Starting scheduler");

            if is_task_pending(&scheduler, &sender_command).await {
                loop {
                    select! {
                        data = receiver.recv() => {
                            if let Ok((topic, qos, retain, payload)) = data {
                                mqtt_service
                                    .lock()
                                    .await
                                    .publish(MqttPublishEvent::new(topic, qos, retain, payload))
                                    .await;

                                if !is_task_pending(&scheduler, &sender_command).await {
                                    break
                                };
                            }
                        },
                        _ = receiver_exit.recv() => {
                            if let Err(e) = scheduler.lock().await.shutdown().await {
                                println!("Error while shutting down {e:?}");
                            }

                            return;
                        }
                    }
                }
            }

            debug!("Scheduler terminated")
        });

        self.scheduler.lock().await.start().await?;

        Ok(task_handle)
    }

    fn create_job_one_shot(
        initial_delay: &Duration,
        retain: bool,
        qos: QoS,
        payload: &[u8],
        topic: &str,
        sender_data: broadcast::Sender<(String, QoS, bool, Vec<u8>)>,
    ) -> Result<Job, JobSchedulerError> {
        let payload = payload.to_owned();
        let topic = topic.to_owned();

        Job::new_one_shot_async(
            *initial_delay,
            move |_uuid: Uuid, _scheduler: JobScheduler| {
                let payload = payload.clone();
                let pc = sender_data.clone();
                let topic = topic.clone();

                Box::pin(async move {
                    let tx = (topic, qos, retain, payload.clone());
                    let _ = pc.clone().send(tx);
                })
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn create_job_repeated_count(
        contexts: Arc<Mutex<JobContextStorage>>,
        interval: &Duration,
        retain: bool,
        qos: QoS,
        payload: &[u8],
        topic: &str,
        sender_data: broadcast::Sender<(String, QoS, bool, Vec<u8>)>,
        count: u32,
    ) -> Result<Job, JobSchedulerError> {
        let payload = payload.to_owned();
        let topic = topic.to_owned();

        Job::new_repeated_async(*interval, move |uuid: Uuid, scheduler: JobScheduler| {
            let payload = payload.clone();
            let pc = sender_data.clone();
            let topic = topic.clone();
            let contexts = contexts.clone();

            Box::pin(async move {
                if !contexts.lock().await.exists(&uuid) {
                    contexts.lock().await.get_or_create_context(&uuid).count = Some(count);
                }
                let mut counter = contexts
                    .lock()
                    .await
                    .get_or_create_context(&uuid)
                    .count
                    .unwrap();

                let tx = (topic, qos, retain, payload.clone());
                let _ = pc.clone().send(tx);

                counter -= 1;
                contexts.lock().await.get_or_create_context(&uuid).count = Some(counter);

                if counter == 0 {
                    debug!("Removing periodic trigger {}", uuid);
                    contexts.lock().await.remove(&uuid);
                    let _ = scheduler.remove(&uuid).await;
                }
            })
        })
    }

    fn create_job_repeated_forever(
        interval: &Duration,
        retain: bool,
        qos: QoS,
        payload: Vec<u8>,
        topic: &str,
        sender_data: broadcast::Sender<(String, QoS, bool, Vec<u8>)>,
    ) -> Result<Job, JobSchedulerError> {
        let payload = payload.clone();
        let topic = topic.to_owned();

        Job::new_repeated_async(*interval, move |_uuid: Uuid, _scheduler: JobScheduler| {
            let payload = payload.clone();
            let pc = sender_data.clone();
            let topic = topic.clone();

            Box::pin(async move {
                let tx = (topic, qos, retain, payload.clone());
                let _ = pc.clone().send(tx);
            })
        })
    }
}

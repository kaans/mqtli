use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use log::debug;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use uuid::Uuid;

use crate::config::topic::Topic;
use crate::config::PublishInputType;
use crate::mqtt::{MqttService, QoS};
use crate::payload::PayloadFormat;
use crate::publish::TriggerError;

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
    pub receiver: Receiver<(String, QoS, bool, Vec<u8>)>,
    pub publish_channel: Sender<(String, QoS, bool, Vec<u8>)>,
    job_contexts: Arc<Mutex<JobContextStorage>>,
}

impl TriggerPeriodic {
    pub async fn new(mqtt_service: Arc<Mutex<dyn MqttService>>) -> Self {
        let (publish_channel, receiver) = mpsc::channel::<(String, QoS, bool, Vec<u8>)>(32);

        Self {
            scheduler: Arc::new(Mutex::new(
                JobScheduler::new()
                    .await
                    .expect("Could not start job scheduler"),
            )),
            mqtt_service,
            publish_channel,
            receiver,
            job_contexts: Arc::new(Mutex::new(JobContextStorage::new())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_schedule(
        &mut self,
        interval: &Duration,
        count: &Option<u32>,
        initial_delay: &Duration,
        output_topic: &Topic,
        qos: &QoS,
        retain: bool,
        input_type: &PublishInputType,
    ) -> Result<(), TriggerError> {
        let payload: Vec<u8> = match PayloadFormat::new(input_type, output_topic.payload_type()) {
            Ok(payload) => payload.try_into()?,
            Err(e) => return Err(TriggerError::CouldNotConvertPayload(e)),
        };

        let qos = *qos;

        let publish_channel = self.publish_channel.clone();
        let scheduler = self.scheduler.clone();
        let initial_delay = *initial_delay;
        let contexts = self.job_contexts.clone();
        let topic = output_topic.topic().to_owned();
        let count = *count;
        let interval = *interval;

        match count {
            Some(1) => {
                let job = Self::create_job_one_shot(
                    &initial_delay,
                    retain,
                    qos,
                    &payload,
                    &topic,
                    &publish_channel,
                )?;

                scheduler.lock().await.add(job).await?;
            }
            Some(count) => {
                let job_initial = Self::create_job_one_shot(
                    &initial_delay,
                    retain,
                    qos,
                    &payload,
                    &topic,
                    &publish_channel,
                )
                .expect("Could not create job");

                scheduler
                    .lock()
                    .await
                    .add(job_initial)
                    .await
                    .expect("Could not add job");

                task::spawn(async move {
                    tokio::time::sleep(initial_delay).await;

                    let job_repeated = Self::create_job_repeated_count(
                        contexts,
                        &interval,
                        retain,
                        qos,
                        &payload,
                        &topic,
                        &publish_channel,
                        count,
                    )
                    .expect("Could not create job");

                    scheduler
                        .lock()
                        .await
                        .add(job_repeated)
                        .await
                        .expect("Could not add job");
                });
            }
            None => {
                let job_initial = Self::create_job_one_shot(
                    &initial_delay,
                    retain,
                    qos,
                    &payload,
                    &topic,
                    &publish_channel,
                )
                .expect("Could not create job");

                scheduler
                    .lock()
                    .await
                    .add(job_initial)
                    .await
                    .expect("Could not add job");

                task::spawn(async move {
                    tokio::time::sleep(initial_delay).await;

                    let job_repeated = Self::create_job_repeated_forever(
                        &interval,
                        retain,
                        qos,
                        payload,
                        &topic,
                        publish_channel,
                    )
                    .expect("Could not create job");

                    scheduler
                        .lock()
                        .await
                        .add(job_repeated)
                        .await
                        .expect("Could not add job");
                });
            }
        };

        Ok(())
    }

    fn create_job_one_shot(
        initial_delay: &Duration,
        retain: bool,
        qos: QoS,
        payload: &[u8],
        topic: &str,
        publish_channel: &Sender<(String, QoS, bool, Vec<u8>)>,
    ) -> Result<Job, JobSchedulerError> {
        let payload = payload.to_owned();
        let topic = topic.to_owned();
        let publish_channel = publish_channel.clone();

        Job::new_one_shot_async(
            *initial_delay,
            move |_uuid: Uuid, _scheduler: JobScheduler| {
                let payload = payload.clone();
                let pc = publish_channel.clone();
                let topic = topic.clone();

                Box::pin(async move {
                    let tx = (topic, qos, retain, payload.clone());
                    let _ = pc.clone().send(tx).await;
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
        publish_channel: &Sender<(String, QoS, bool, Vec<u8>)>,
        count: u32,
    ) -> Result<Job, JobSchedulerError> {
        let payload = payload.to_owned();
        let topic = topic.to_owned();
        let publish_channel = publish_channel.clone();

        Job::new_repeated_async(*interval, move |uuid: Uuid, scheduler: JobScheduler| {
            let payload = payload.clone();
            let pc = publish_channel.clone();
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
                let _ = pc.clone().send(tx).await;

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
        publish_channel: Sender<(String, QoS, bool, Vec<u8>)>,
    ) -> Result<Job, JobSchedulerError> {
        let payload = payload.clone();
        let topic = topic.to_owned();
        let publish_channel = publish_channel.clone();

        Job::new_repeated_async(*interval, move |_uuid: Uuid, _scheduler: JobScheduler| {
            let payload = payload.clone();
            let pc = publish_channel.clone();
            let topic = topic.clone();

            Box::pin(async move {
                let tx = (topic, qos, retain, payload.clone());
                let _ = pc.clone().send(tx).await;
            })
        })
    }

    pub async fn start(self) -> Result<(), TriggerError> {
        let mut receiver = self.receiver;
        let mqtt_service = self.mqtt_service.clone();

        task::spawn(async move {
            loop {
                if let Some((topic, qos, retain, payload)) = receiver.recv().await {
                    mqtt_service
                        .lock()
                        .await
                        .publish(topic, qos, retain, payload)
                        .await;
                }
            }
        });

        Ok(self.scheduler.lock().await.start().await?)
    }
}

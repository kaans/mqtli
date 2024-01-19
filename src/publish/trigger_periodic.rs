use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use log::debug;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::config::{PayloadType, PublishInputType};
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
            self.contexts.insert(uuid.clone(), context);
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
    scheduler: JobScheduler,
    mqtt_service: Arc<Mutex<dyn MqttService>>,
    pub receiver: Receiver<(String, QoS, bool, Vec<u8>)>,
    pub publish_channel: Sender<(String, QoS, bool, Vec<u8>)>,
    job_contexts: Arc<Mutex<JobContextStorage>>,
}

impl TriggerPeriodic {
    pub async fn new(mqtt_service: Arc<Mutex<dyn MqttService>>) -> Self {
        let (publish_channel, receiver) = mpsc::channel::<(String, QoS, bool, Vec<u8>)>(32);

        Self {
            scheduler: JobScheduler::new()
                .await
                .expect("Could not start job scheduler"),
            mqtt_service,
            publish_channel,
            receiver,
            job_contexts: Arc::new(Mutex::new(JobContextStorage::new())),
        }
    }

    pub async fn add_schedule(
        &mut self,
        interval: &Duration,
        count: &Option<u32>,
        initial_delay: &Duration,
        topic: &str,
        qos: &QoS,
        retain: bool,
        input: &PublishInputType,
        output_format: &PayloadType,
    ) -> Result<Uuid, TriggerError> {
        let payload: Vec<u8> = match PayloadFormat::new(input, output_format) {
            Ok(payload) => payload.try_into()?,
            Err(e) => return Err(TriggerError::CouldNotConvertPayload(e)),
        };

        let qos = *qos;

        let payload = payload.clone();
        let topic = String::from(topic);
        let publish_channel = self.publish_channel.clone();

        let job = match count {
            Some(1) => Job::new_one_shot_async(
                initial_delay.clone(),
                move |_uuid: Uuid, _scheduler: JobScheduler| {
                    let payload = payload.clone();
                    let pc = publish_channel.clone();
                    let topic = topic.clone();

                    Box::pin(async move {
                        let tx = (topic, qos, retain, payload.clone());
                        let _ = pc.clone().send(tx).await;
                    })
                },
            )?,
            Some(count) => {
                let contexts = self.job_contexts.clone();
                let count = count.clone();

                Job::new_repeated_async(
                    interval.clone(),
                    move |uuid: Uuid, scheduler: JobScheduler| {
                        let payload = payload.clone();
                        let pc = publish_channel.clone();
                        let topic = topic.clone();
                        let contexts = contexts.clone();
                        let count = count.clone();

                        Box::pin(async move {
                            if !contexts.lock().await.exists(&uuid) {
                                contexts.lock().await.get_or_create_context(&uuid).count =
                                    Some(count.clone());
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
                            contexts.lock().await.get_or_create_context(&uuid).count =
                                Some(counter);

                            if counter == 0 {
                                debug!("Removing periodic trigger {}", uuid);
                                contexts.lock().await.remove(&uuid);
                                let _ = scheduler.remove(&uuid).await;
                            }
                        })
                    },
                )?
            }
            None => Job::new_repeated_async(
                interval.clone(),
                move |_uuid: Uuid, _scheduler: JobScheduler| {
                    let payload = payload.clone();
                    let pc = publish_channel.clone();
                    let topic = topic.clone();

                    Box::pin(async move {
                        let tx = (topic, qos, retain, payload.clone());
                        let _ = pc.clone().send(tx).await;
                    })
                },
            )?,
        };

        Ok(self.scheduler.add(job).await?)
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

        Ok(self.scheduler.start().await?)
    }
}

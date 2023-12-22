use std::sync::Arc;
use std::time::Duration;

use clokwerk::{AsyncScheduler, Interval, Job};
use log::info;
use rumqttc::v5::mqttbytes::QoS;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::{PayloadType, PublishInputType};
use crate::mqtt_service::MqttService;
use crate::payload::PayloadFormat;
use crate::publish::TriggerError;

pub struct TriggerPeriodic {
    scheduler: Option<Box<AsyncScheduler>>,
    task_handle: Option<JoinHandle<()>>,
    mqtt_service: Arc<Mutex<MqttService>>,
}

impl TriggerPeriodic {
    pub fn new(mqtt_service: Arc<Mutex<MqttService>>) -> Self {
        Self {
            scheduler: Option::from(Box::new(AsyncScheduler::new())),
            task_handle: None,
            mqtt_service,
        }
    }

    pub fn add_schedule(
        &mut self,
        interval: &Duration,
        count: &Option<u32>,
        initial_delay: &Duration,
        topic: &str,
        qos: &QoS,
        retain: bool,
        input: &PublishInputType,
        output_format: &PayloadType,
    ) -> Result<(), TriggerError> {
        if let Some(scheduler) = self.scheduler.as_mut() {
            let mut job = scheduler
                .every(Interval::Seconds(interval.as_secs() as u32))
                .plus(Interval::Seconds(initial_delay.as_secs() as u32));

            if let Some(count) = count {
                job = job.count(*count as usize);
            } else {
                job = job.forever();
            }

            let payload: Vec<u8> = match PayloadFormat::new(input, output_format) {
                Ok(payload) => payload.try_into()?,
                Err(e) => return Err(TriggerError::CouldNotConvertPayload(e)),
            };

            let mqtt_service = Arc::clone(&self.mqtt_service);
            let topic = String::from(topic);
            let qos = *qos;

            let clos = move || {
                let topic = String::from(topic.as_str());
                let payload: Vec<u8> = payload.clone();
                let mqtt_service = Arc::clone(&mqtt_service);

                async move {
                    let a = &mut mqtt_service.lock().await;

                    a.publish(topic, qos, retain, payload).await;
                }
            };

            job.run(clos);

            Ok(())
        } else {
            Err(TriggerError::SchedulerAlreadyRunning)
        }
    }

    pub async fn start(mut self) {
        if self.scheduler.is_some() {
            let mut scheduler = self.scheduler.unwrap();

            self.task_handle = Some(tokio::spawn(async move {
                loop {
                    scheduler.run_pending().await;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }));
        } else {
            info!("Not starting scheduler, not jobs scheduled");
        }
    }
}

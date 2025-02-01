use mqtlib::config::publish::PublishTriggerType::Periodic;
use mqtlib::config::subscription::Subscription;
use mqtlib::config::topic::TopicStorage;
use mqtlib::mqtt::{MqttReceiveEvent, MqttService};
use mqtlib::payload::{PayloadFormat, PayloadFormatError};
use mqtlib::publish::trigger_periodic::{Command, TriggerPeriodic};
use mqtlib::publish::TriggerError;
use rumqttc::v5::Incoming;
use rumqttc::Incoming as IncomingV311;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

pub fn start_scheduler_monitor_task(
    mqtt_service_publish: Arc<Mutex<dyn MqttService>>,
    mut receiver_command: Receiver<Command>,
    filtered_subscriptions_command: Vec<(Subscription, String)>,
) {
    tokio::spawn(async move {
        match receiver_command.recv().await {
            Ok(Command::NoMoreTasksPending) => {
                if filtered_subscriptions_command.is_empty() {
                    debug!("No more pending tasks and no subscriptions, disconnecting from MQTT broker");
                    let _ = mqtt_service_publish.lock().await.disconnect().await;
                }
            }
            Err(e) => {
                debug!("Received error from scheduler, disconnecting: {e:?}");
                let _ = mqtt_service_publish.lock().await.disconnect().await;
            }
        }
    });
}

pub fn start_scheduler_task(
    scheduler: TriggerPeriodic,
    sender: Sender<MqttReceiveEvent>,
    topics: Arc<TopicStorage>,
    receiver_exit: Receiver<()>,
) {
    let mut receiver_connect = sender.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver_connect.recv().await {
            match event {
                MqttReceiveEvent::V5(rumqttc::v5::Event::Incoming(Incoming::ConnAck(_)))
                | MqttReceiveEvent::V311(rumqttc::Event::Incoming(IncomingV311::ConnAck(_))) => {
                    info!("Connected to broker");

                    let _ = start_scheduler(topics.clone(), scheduler, receiver_exit).await;

                    return;
                }
                _ => {}
            }
        }
    });
}

async fn start_scheduler(
    topic_storage: Arc<TopicStorage>,
    mut scheduler: TriggerPeriodic,
    receiver_exit: Receiver<()>,
) -> Result<JoinHandle<()>, TriggerError> {
    for topic in topic_storage.topics.iter() {
        if let Some(publish) = topic
            .publish()
            .as_ref()
            .filter(|publish| *publish.enabled())
        {
            let topic_str = topic.topic().to_owned();
            for trigger in publish.trigger() {
                #[allow(irrefutable_let_patterns)]
                if let Periodic(value) = trigger {
                    match PayloadFormat::try_from(publish.input())
                        .and_then(|data| {
                            publish
                                .apply_filters(data)
                                .map_err(PayloadFormatError::from)
                        })
                        .and_then(|data| {
                            data.into_iter()
                                .map(|a| {
                                    let b = PayloadFormat::try_from((a, topic.payload_type()));
                                    b
                                })
                                .collect::<Result<Vec<PayloadFormat>, PayloadFormatError>>()
                        })
                        .and_then(|data| {
                            data.into_iter()
                                .map(|payload| payload.try_into())
                                .collect::<Result<Vec<Vec<u8>>, PayloadFormatError>>()
                        }) {
                        Ok(val) => {
                            for data in val {
                                if let Err(e) = scheduler
                                    .add_schedule(
                                        value.interval(),
                                        value.count(),
                                        value.initial_delay(),
                                        &topic_str,
                                        publish.qos(),
                                        *publish.retain(),
                                        data,
                                    )
                                    .await
                                {
                                    error!("Error while adding schedule: {}", e);
                                };
                            }
                        }
                        Err(e) => {
                            error!("Error while converting payload: {e}");
                        }
                    };
                }
            }
        }
    }

    scheduler.start(receiver_exit).await
}

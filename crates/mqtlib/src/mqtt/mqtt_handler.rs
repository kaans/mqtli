use std::sync::Arc;

use rumqttc::v5::mqttbytes::v5::PublishProperties;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task;
use tokio::task::JoinHandle;
use tracing::error;

use crate::config::topic::TopicStorage;
use crate::mqtt::{MessageEvent, MessageReceivedData, MqttReceiveEvent, QoS};
use crate::payload::PayloadFormat;

pub struct MqttHandler {
    task_handle: Option<JoinHandle<()>>,
    topic_storage: Arc<TopicStorage>,
}

impl MqttHandler {
    pub fn new(topic_storage: Arc<TopicStorage>) -> MqttHandler {
        MqttHandler {
            task_handle: None,
            topic_storage,
        }
    }

    pub fn start_task(
        &mut self,
        mut receiver: Receiver<MqttReceiveEvent>,
        sender_message: Sender<MessageEvent>,
    ) {
        let topic_storage = self.topic_storage.clone();

        self.task_handle = Some(task::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                MqttHandler::handle_event(event, &topic_storage, &sender_message);
            }
        }));
    }

    pub async fn await_task(self) {
        if self.task_handle.is_some() {
            self.task_handle
                .unwrap()
                .await
                .expect("Could not join thread");
        }
    }

    pub fn handle_event(
        event: MqttReceiveEvent,
        topic_storage: &Arc<TopicStorage>,
        sender_message: &Sender<MessageEvent>,
    ) {
        match event {
            MqttReceiveEvent::V5(event) => {
                v5::handle_event(event, topic_storage, sender_message);
            }
            MqttReceiveEvent::V311(event) => {
                v311::handle_event(event, topic_storage, sender_message);
            }
        }
    }

    fn handle_incoming_message(
        topic_storage: &Arc<TopicStorage>,
        incoming_value: Vec<u8>,
        incoming_topic_str: &str,
        qos: QoS,
        retain: bool,
        _option: Option<PublishProperties>,
        sender_message: &Sender<MessageEvent>,
    ) {
        topic_storage
            .topics
            .iter()
            .filter(|topic| topic.contains(incoming_topic_str))
            .filter_map(|topic| {
                topic
                    .subscription()
                    .as_ref()
                    .map(|subscription| (subscription, topic.payload_type()))
            })
            .filter(|(subscription, _)| *subscription.enabled())
            .for_each(|(subscription, payload_type)| {
                let result =
                    PayloadFormat::try_from((payload_type.clone(), incoming_value.clone()));

                match result {
                    Ok(content) => {
                        if sender_message
                            .send(MessageEvent::ReceivedUnfiltered(MessageReceivedData {
                                topic: incoming_topic_str.into(),
                                qos,
                                retain,
                                payload: content.clone(),
                            }))
                            .is_err()
                        {
                            //ignore, no receiver is listening
                        }

                        match subscription.apply_filters(content.clone()) {
                            Ok(content) => {
                                content.iter().for_each(|content| {
                                    if sender_message
                                        .send(MessageEvent::ReceivedFiltered(MessageReceivedData {
                                            topic: incoming_topic_str.into(),
                                            qos,
                                            retain,
                                            payload: content.clone(),
                                        }))
                                        .is_err()
                                    {
                                        //ignore, no receiver is listening
                                    }
                                })
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                };
            })
    }
}

mod v5 {
    use crate::config::topic::TopicStorage;
    use crate::mqtt::mqtt_handler::MqttHandler;
    use crate::mqtt::{MessageEvent, QoS};
    use std::str::from_utf8;
    use std::sync::Arc;
    use tokio::sync::broadcast::Sender;
    use tracing::debug;

    pub fn handle_event(
        event: rumqttc::v5::Event,
        topic_storage: &Arc<TopicStorage>,
        sender_message: &Sender<MessageEvent>,
    ) {
        match event {
            rumqttc::v5::Event::Incoming(event) => {
                if let rumqttc::v5::Incoming::Publish(value) = event {
                    let incoming_topic =
                        from_utf8(value.topic.as_ref()).expect("Topic is not in UTF-8 format");
                    let qos = QoS::from(value.qos);

                    debug!(
                        "Incoming message on topic {} (QoS: {})",
                        incoming_topic, qos
                    );

                    MqttHandler::handle_incoming_message(
                        topic_storage,
                        value.payload.to_vec(),
                        incoming_topic,
                        qos,
                        value.retain,
                        value.properties,
                        sender_message,
                    );
                }
            }
            rumqttc::v5::Event::Outgoing(_event) => {}
        }
    }
}

mod v311 {
    use crate::config::topic::TopicStorage;
    use crate::mqtt::mqtt_handler::MqttHandler;
    use crate::mqtt::{MessageEvent, QoS};
    use std::str::from_utf8;
    use std::sync::Arc;
    use tokio::sync::broadcast::Sender;
    use tracing::debug;

    pub fn handle_event(
        event: rumqttc::Event,
        topic_storage: &Arc<TopicStorage>,
        sender_message: &Sender<MessageEvent>,
    ) {
        match event {
            rumqttc::Event::Incoming(event) => {
                if let rumqttc::Incoming::Publish(value) = event {
                    let incoming_topic =
                        from_utf8(value.topic.as_ref()).expect("Topic is not in UTF-8 format");
                    let qos = QoS::from(value.qos);

                    debug!(
                        "Incoming message on topic {} (QoS: {})",
                        incoming_topic, qos
                    );

                    MqttHandler::handle_incoming_message(
                        topic_storage,
                        value.payload.to_vec(),
                        incoming_topic,
                        qos,
                        value.retain,
                        None,
                        sender_message,
                    );
                }
            }
            rumqttc::Event::Outgoing(_event) => {}
        }
    }
}

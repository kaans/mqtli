use std::str::from_utf8;

use log::{debug, info};
use rumqttc::v5::{Event, Incoming};
use tokio::sync::broadcast::Receiver;
use tokio::task;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::{PayloadType, Topic};
use crate::payload::protobuf::PayloadProtobufHandler;
use crate::payload::text::PayloadTextHandler;

pub struct MqttHandler {
    task_handle: Option<JoinHandle<()>>,
    topics: Vec<Topic>,
}

impl MqttHandler {
    pub fn new(topics: &Vec<Topic>) -> MqttHandler {
        MqttHandler {
            task_handle: None,
            topics: topics.clone(),
        }
    }

    pub(crate) fn start_task(&mut self, mut receiver: Receiver<Event>) {
        let topics = self.topics.clone();

        self.task_handle = Some(task::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        MqttHandler::handle_event(event, &topics);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }));
    }

    pub async fn await_task(self) {
        if self.task_handle.is_some() {
            self.task_handle.unwrap().await.expect("Could not join thread");
        }
    }

    pub fn handle_event(event: Event, topics: &Vec<Topic>) {
        match event {
            Event::Incoming(event) => {
                match event {
                    Incoming::Publish(value) => {
                        let incoming_topic = from_utf8(value.topic.as_ref()).unwrap();

                        info!("Incoming message on topic {} (QoS: {:?})",
                                            incoming_topic,
                                            value.qos);

                        for topic in topics {
                            if topic.topic() == incoming_topic {
                                match topic.payload() {
                                    PayloadType::Text(_) => {
                                        debug!("Handling text payload of topic {}", incoming_topic);
                                        PayloadTextHandler::handle_publish(&value);
                                    }
                                    PayloadType::Protobuf(payload) => {
                                        debug!("Handling protobuf payload of topic {}", incoming_topic);
                                        PayloadProtobufHandler::handle_publish(&value, payload.definition(), payload.message());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Outgoing(_event) => {}
        }
    }
}
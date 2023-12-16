use std::str::from_utf8;
use std::sync::Arc;

use log::{debug, error, info};
use rumqttc::v5::{Event, Incoming};
use tokio::sync::broadcast::Receiver;
use tokio::task;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::{OutputTarget, PayloadType, Topic};
use crate::config::mqtli_config::OutputTarget::Console;
use crate::output::console::ConsoleOutput;
use crate::output::file::FileOutput;
use crate::payload::protobuf::protobuf::PayloadProtobufHandler;
use crate::payload::text::PayloadTextHandler;

pub struct MqttHandler {
    task_handle: Option<JoinHandle<()>>,
    topics: Arc<Box<Vec<Topic>>>,
}

impl MqttHandler {
    pub fn new(topics: Arc<Box<Vec<Topic>>>) -> MqttHandler {
        MqttHandler {
            task_handle: None,
            topics,
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
                                for output in topic.outputs() {
                                    let result = match topic.payload() {
                                        PayloadType::Text(_) => {
                                            debug!("Handling text payload of topic {} with format {:?}", incoming_topic, output.format());
                                            PayloadTextHandler::handle_publish(&value, output.format())
                                        }
                                        PayloadType::Protobuf(payload) => {
                                            debug!("Handling protobuf payload of topic {} with format {:?}", incoming_topic, output.format());
                                            PayloadProtobufHandler::handle_publish(&value, payload.definition(), payload.message(), output.format())
                                        }
                                    };

                                    match result {
                                        Ok(content) => {
                                            let result = match output.target() {
                                                Console(_options) => {
                                                    ConsoleOutput::output(content)
                                                }
                                                OutputTarget::File(file) => {
                                                    FileOutput::output(content, file)
                                                }
                                            };

                                            if let Err(e) = result {
                                                error!("{:?}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("{:?}", e);
                                        }
                                    };
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
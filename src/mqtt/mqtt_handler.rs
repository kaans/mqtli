use std::sync::Arc;

use log::error;
use tokio::sync::broadcast::Receiver;
use tokio::task;
use tokio::task::JoinHandle;

use crate::config::subscription::{Output, OutputTarget};
use crate::config::topic::Topic;
use crate::mqtt::{MqttEvent, QoS};
use crate::output::console::ConsoleOutput;
use crate::output::file::FileOutput;
use crate::output::OutputError;
use crate::payload::PayloadFormat;

pub struct MqttHandler {
    task_handle: Option<JoinHandle<()>>,
    topics: Arc<Vec<Topic>>,
}

impl MqttHandler {
    pub fn new(topics: Arc<Vec<Topic>>) -> MqttHandler {
        MqttHandler {
            task_handle: None,
            topics,
        }
    }

    pub fn start_task(&mut self, mut receiver: Receiver<MqttEvent>) {
        let topics = self.topics.clone();

        self.task_handle = Some(task::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                MqttHandler::handle_event(event, &topics);
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

    pub fn handle_event(event: MqttEvent, topics: &[Topic]) {
        match event {
            MqttEvent::V5(event) => {
                v5::handle_event(event, topics);
            }
            MqttEvent::V311(event) => {
                v311::handle_event(event, topics);
            }
        }
    }

    fn handle_incoming_message(
        topics: &[Topic],
        incoming_value: Vec<u8>,
        incoming_topic_str: &str,
        qos: QoS,
        retain: bool,
    ) {
        topics
            .iter()
            .filter(|topic| topic.contains(incoming_topic_str) && *topic.subscription().enabled())
            .for_each(|incoming_topic| {
                for output in incoming_topic.subscription().outputs() {
                    let result = PayloadFormat::try_from((
                        incoming_topic.payload_type().clone(),
                        incoming_value.clone(),
                    ));

                    match result {
                        Ok(content) => match incoming_topic.subscription().apply_filters(content) {
                            Ok(content) => content.iter().for_each(|content| {
                                if let Err(e) = Self::forward_to_output(
                                    output,
                                    incoming_topic_str,
                                    content.clone(),
                                    qos,
                                    retain,
                                ) {
                                    error!("{}", e);
                                }
                            }),
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        },
                        Err(e) => {
                            error!("{}", e);
                        }
                    };
                }
            })
    }

    fn forward_to_output(
        output: &Output,
        topic: &str,
        content: PayloadFormat,
        qos: QoS,
        retain: bool,
    ) -> Result<(), OutputError> {
        let conv = PayloadFormat::try_from((content, output.format()))?;

        let result = match output.target() {
            OutputTarget::Console(_options) => {
                ConsoleOutput::output(topic, conv.clone().try_into()?, conv, qos, retain)
            }
            OutputTarget::File(file) => FileOutput::output(conv.try_into()?, file),
        };

        result
    }
}

mod v5 {
    use std::str::from_utf8;

    use log::info;

    use crate::config::topic::Topic;
    use crate::mqtt::mqtt_handler::MqttHandler;
    use crate::mqtt::QoS;

    pub fn handle_event(event: rumqttc::v5::Event, topics: &[Topic]) {
        match event {
            rumqttc::v5::Event::Incoming(event) => {
                if let rumqttc::v5::Incoming::Publish(value) = event {
                    let incoming_topic =
                        from_utf8(value.topic.as_ref()).expect("Topic is not in UTF-8 format");
                    let qos = QoS::from(value.qos);

                    info!(
                        "Incoming message on topic {} (QoS: {})",
                        incoming_topic, qos
                    );

                    MqttHandler::handle_incoming_message(
                        topics,
                        value.payload.to_vec(),
                        incoming_topic,
                        qos,
                        value.retain,
                    );
                }
            }
            rumqttc::v5::Event::Outgoing(_event) => {}
        }
    }
}

mod v311 {
    use std::str::from_utf8;

    use log::info;

    use crate::config::topic::Topic;
    use crate::mqtt::mqtt_handler::MqttHandler;
    use crate::mqtt::QoS;

    pub fn handle_event(event: rumqttc::Event, topics: &[Topic]) {
        match event {
            rumqttc::Event::Incoming(event) => {
                if let rumqttc::Incoming::Publish(value) = event {
                    let incoming_topic =
                        from_utf8(value.topic.as_ref()).expect("Topic is not in UTF-8 format");
                    let qos = QoS::from(value.qos);

                    info!(
                        "Incoming message on topic {} (QoS: {})",
                        incoming_topic, qos
                    );

                    MqttHandler::handle_incoming_message(
                        topics,
                        value.payload.to_vec(),
                        incoming_topic,
                        qos,
                        value.retain,
                    );
                }
            }
            rumqttc::Event::Outgoing(_event) => {}
        }
    }
}

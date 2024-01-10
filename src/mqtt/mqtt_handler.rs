use std::sync::Arc;

use log::error;
use tokio::sync::broadcast::Receiver;
use tokio::task;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::OutputTarget::Console;
use crate::config::mqtli_config::{Output, OutputTarget, Topic};
use crate::mqtt::MqttEvent;
use crate::output::console::ConsoleOutput;
use crate::output::file::FileOutput;
use crate::output::OutputError;
use crate::payload::{PayloadFormat, PayloadFormatTopic};

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

    pub(crate) fn start_task(&mut self, mut receiver: Receiver<MqttEvent>) {
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

    pub fn handle_event(event: MqttEvent, topics: &Vec<Topic>) {
        match event {
            MqttEvent::V5(event) => {
                v5::handle_event(event, topics);
            }
            MqttEvent::V311(event) => {
                v311::handle_event(event, topics);
            }
        }
    }

    fn handle_incoming_message(topics: &Vec<Topic>, value: Vec<u8>, incoming_topic: &str) {
        for topic in topics {
            if topic.topic() == incoming_topic && *topic.subscription().enabled() {
                for output in topic.subscription().outputs() {
                    let result = PayloadFormat::try_from(PayloadFormatTopic::new(
                        topic.payload().clone(),
                        value.clone(),
                    ));

                    match result {
                        Ok(content) => {
                            if let Err(e) = Self::forward_to_output(output, content) {
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

    fn forward_to_output(output: &Output, content: PayloadFormat) -> Result<(), OutputError> {
        let conv = PayloadFormat::try_from((content, output.format()))?;

        let result = match output.target() {
            Console(_options) => ConsoleOutput::output(conv.try_into()?),
            OutputTarget::File(file) => FileOutput::output(conv.try_into()?, file),
        };

        result
    }
}

mod v5 {
    use std::str::from_utf8;

    use log::info;
    use rumqttc::v5::{Event, Incoming};

    use crate::config::mqtli_config::Topic;
    use crate::mqtt::mqtt_handler::MqttHandler;

    pub fn handle_event(event: Event, topics: &Vec<Topic>) {
        match event {
            Event::Incoming(event) => {
                if let Incoming::Publish(value) = event {
                    let incoming_topic = from_utf8(value.topic.as_ref()).unwrap();

                    info!(
                        "Incoming message on topic {} (QoS: {:?})",
                        incoming_topic, value.qos
                    );

                    MqttHandler::handle_incoming_message(
                        topics,
                        value.payload.to_vec(),
                        incoming_topic,
                    );
                }
            }
            Event::Outgoing(_event) => {}
        }
    }
}

mod v311 {
    use std::str::from_utf8;

    use log::info;

    use crate::config::mqtli_config::Topic;
    use crate::mqtt::mqtt_handler::MqttHandler;

    pub fn handle_event(event: rumqttc::Event, topics: &Vec<Topic>) {
        match event {
            rumqttc::Event::Incoming(event) => {
                if let rumqttc::Incoming::Publish(value) = event {
                    let incoming_topic = from_utf8(value.topic.as_ref()).unwrap();

                    info!(
                        "Incoming message on topic {} (QoS: {:?})",
                        incoming_topic, value.qos
                    );

                    MqttHandler::handle_incoming_message(
                        topics,
                        value.payload.to_vec(),
                        incoming_topic,
                    );
                }
            }
            rumqttc::Event::Outgoing(_event) => {}
        }
    }
}

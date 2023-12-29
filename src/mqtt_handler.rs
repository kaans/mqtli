use std::str::from_utf8;
use std::sync::Arc;

use log::{debug, error, info};
use rumqttc::v5::{Event, Incoming};
use tokio::sync::broadcast::Receiver;
use tokio::task;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::OutputTarget::Console;
use crate::config::mqtli_config::{Output, OutputTarget, Topic};
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

    pub(crate) fn start_task(&mut self, mut receiver: Receiver<Event>) {
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

    pub fn handle_event(event: Event, topics: &Vec<Topic>) {
        match event {
            Event::Incoming(event) => {
                if let Incoming::Publish(value) = event {
                    let incoming_topic = from_utf8(value.topic.as_ref()).unwrap();

                    info!(
                        "Incoming message on topic {} (QoS: {:?})",
                        incoming_topic, value.qos
                    );

                    for topic in topics {
                        if topic.topic() == incoming_topic && *topic.subscription().enabled() {
                            for output in topic.subscription().outputs() {
                                let result = PayloadFormat::try_from(PayloadFormatTopic::new(
                                    topic.payload().clone(),
                                    value.payload.to_vec(),
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
            }
            Event::Outgoing(_event) => {}
        }
    }

    fn forward_to_output(output: &Output, content: PayloadFormat) -> Result<(), OutputError> {
        let conv = content.convert_for_output(output)?;

        let result = match output.target() {
            Console(_options) => ConsoleOutput::output(conv.try_into()?),
            OutputTarget::File(file) => FileOutput::output(conv.try_into()?, file),
        };

        result
    }
}

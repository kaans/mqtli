use std::sync::Arc;

use log::error;
use rumqttc::v5::mqttbytes::v5::PublishProperties;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task;
use tokio::task::JoinHandle;

use crate::config::subscription::{Output, OutputTarget};
use crate::config::topic::Topic;
use crate::mqtt::{MessageEvent, MessagePublishData, MessageReceivedData, MqttReceiveEvent, QoS};
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

    pub fn start_task(
        &mut self,
        mut receiver: Receiver<MqttReceiveEvent>,
        sender_message: Sender<MessageEvent>,
    ) {
        let topics = self.topics.clone();

        self.task_handle = Some(task::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                MqttHandler::handle_event(event, &topics, &sender_message);
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
        topics: &[Topic],
        sender_message: &Sender<MessageEvent>,
    ) {
        match event {
            MqttReceiveEvent::V5(event) => {
                v5::handle_event(event, topics, sender_message);
            }
            MqttReceiveEvent::V311(event) => {
                v311::handle_event(event, topics, sender_message);
            }
        }
    }

    fn handle_incoming_message(
        topics: &[Topic],
        incoming_value: Vec<u8>,
        incoming_topic_str: &str,
        qos: QoS,
        retain: bool,
        _option: Option<PublishProperties>,
        sender_message: &Sender<MessageEvent>,
    ) {
        topics
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
                for output in subscription.outputs() {
                    let result =
                        PayloadFormat::try_from((payload_type.clone(), incoming_value.clone()));

                    match result {
                        Ok(content) => match subscription.apply_filters(content) {
                            Ok(content) => content.iter().for_each(|content| {
                                if let Ok(payload) =
                                    PayloadFormat::try_from((content.clone(), output.format()))
                                {
                                    if let Ok(payload) = payload.try_into() {
                                        if sender_message
                                            .send(MessageEvent::Received(MessageReceivedData {
                                                topic: incoming_topic_str.into(),
                                                qos,
                                                retain,
                                                payload,
                                            }))
                                            .is_err()
                                        {
                                            //ignore, no receiver is listening
                                        }
                                    } else {
                                        error!("Could not convert payload");
                                    }
                                } else {
                                    error!("Could not convert payload");
                                }

                                if let Err(e) = Self::forward_to_output(
                                    output,
                                    incoming_topic_str,
                                    content.clone(),
                                    qos,
                                    retain,
                                    sender_message,
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
        sender_message: &Sender<MessageEvent>,
    ) -> Result<(), OutputError> {
        let conv = PayloadFormat::try_from((content, output.format()))?;

        let result = match output.target() {
            OutputTarget::Console(_options) => {
                ConsoleOutput::output(topic, conv.clone().try_into()?, conv, qos, retain)
            }
            OutputTarget::File(file) => FileOutput::output(conv.try_into()?, file),
            OutputTarget::Topic(options) => {
                sender_message
                    .send(MessageEvent::Publish(MessagePublishData::new(
                        options.topic().clone(),
                        *options.qos(),
                        *options.retain(),
                        conv.try_into()?,
                    )))
                    .map_err(OutputError::SendError)?;
                Ok(())
            }
            OutputTarget::Null => Ok(()),
        };

        result
    }
}

mod v5 {
    use crate::config::topic::Topic;
    use crate::mqtt::mqtt_handler::MqttHandler;
    use crate::mqtt::{MessageEvent, QoS};
    use log::info;
    use std::str::from_utf8;
    use tokio::sync::broadcast::Sender;

    pub fn handle_event(
        event: rumqttc::v5::Event,
        topics: &[Topic],
        sender_message: &Sender<MessageEvent>,
    ) {
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
    use crate::config::topic::Topic;
    use crate::mqtt::mqtt_handler::MqttHandler;
    use crate::mqtt::{MessageEvent, QoS};
    use log::info;
    use std::str::from_utf8;
    use tokio::sync::broadcast::Sender;

    pub fn handle_event(
        event: rumqttc::Event,
        topics: &[Topic],
        sender_message: &Sender<MessageEvent>,
    ) {
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
                        None,
                        sender_message,
                    );
                }
            }
            rumqttc::Event::Outgoing(_event) => {}
        }
    }
}

//! # MQTli
//! A multi-payload format CLI tool for communication with an MQTT broker.
//!
//! Main features:
//! - support of many payload formats (json, yaml, protobuf, hex, base64, utf-8, raw)
//! - convert seamlessly between different payload formats (e.g. from json to protobuf)
//! - automatically publish messages using triggers (periodically, messages on topics)
//! - subscribe to topics and output messages to console or to file
//! - support of multiple inputs and outputs per topic
//! - configuration is stored in a file to support complex configuration scenarios and share them
//!

mod args;
mod built_info;

use std::sync::Arc;

use crate::args::load_config;
use anyhow::Context;
use log::{debug, error, info, warn};
use mqtlib::config::mqtli_config::MqttVersion;
use mqtlib::config::publish::PublishTriggerType::Periodic;
use mqtlib::config::subscription::{Output, OutputTarget, Subscription};
use mqtlib::config::topic::TopicStorage;
use mqtlib::mqtt::mqtt_handler::MqttHandler;
use mqtlib::mqtt::v311::mqtt_service::MqttServiceV311;
use mqtlib::mqtt::v5::mqtt_service::MqttServiceV5;
use mqtlib::mqtt::{
    MessageEvent, MessagePublishData, MessageReceivedData, MqttReceiveEvent, MqttService,
};
use mqtlib::output::console::ConsoleOutput;
use mqtlib::output::file::FileOutput;
use mqtlib::output::OutputError;
use mqtlib::payload::PayloadFormat;
use mqtlib::payload::PayloadFormatError;
use mqtlib::publish::trigger_periodic::{Command, TriggerPeriodic};
use mqtlib::publish::TriggerError;
use mqtlib::sparkplug::SparkplugNetwork;
use rumqttc::v5::Incoming;
use rumqttc::Incoming as IncomingV311;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio::{signal, task};
use tracing::Level;
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::util::{SubscriberInitExt, TryInitError};

type ExitCommand = ();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config()?;

    init_logger(config.log_level)?;

    info!(
        "MQTli {} version {} starting",
        config.mode,
        built_info::PKG_VERSION
    );

    debug!("{}", config);

    let (sender_exit, _) = broadcast::channel::<ExitCommand>(5);

    let mqtt_service: Arc<Mutex<dyn MqttService>> = match config.broker().mqtt_version() {
        MqttVersion::V311 => Arc::new(Mutex::new(MqttServiceV311::new(Arc::new(
            config.broker().clone(),
        )))),
        MqttVersion::V5 => Arc::new(Mutex::new(MqttServiceV5::new(Arc::new(
            config.broker().clone(),
        )))),
    };

    let filtered_subscriptions: Vec<(Subscription, String)> = config
        .topic_storage
        .topics
        .iter()
        .filter_map(|topic| {
            topic
                .subscription()
                .clone()
                .map(|s| (s, topic.topic().clone()))
        })
        .filter(|(s, _)| *s.enabled())
        .collect();

    let (sender_receive, _) = broadcast::channel::<MqttReceiveEvent>(32);
    let (sender_message, _) = broadcast::channel::<MessageEvent>(32);

    let topic_storage = Arc::new(config.topic_storage);
    //let topics = Arc::new(config.topic_storage.topics);

    let mqtt_loop_handle = mqtt_service
        .lock()
        .await
        .connect(sender_receive.clone(), sender_exit.subscribe())
        .await
        .with_context(|| "Error while connecting to mqtt broker")?;

    start_publish_task(sender_message.subscribe(), mqtt_service.clone());

    let scheduler = TriggerPeriodic::new(mqtt_service.clone()).await;

    start_scheduler_monitor_task(
        mqtt_service.clone(),
        scheduler.get_receiver_command(),
        filtered_subscriptions.clone(),
    );

    start_scheduler_task(
        scheduler,
        sender_receive.clone(),
        topic_storage.clone(),
        sender_exit.subscribe(),
    );

    let mut incoming_messages_handler = MqttHandler::new(topic_storage.clone());
    incoming_messages_handler.start_task(sender_receive.subscribe(), sender_message.clone());

    start_subscription_task(mqtt_service, sender_receive, filtered_subscriptions);

    let sparkplug_network = Arc::new(Mutex::new(SparkplugNetwork::default()));
    start_sparkplug_monitor(sparkplug_network, sender_message.subscribe());

    start_output_task(
        sender_message.subscribe(),
        topic_storage.clone(),
        sender_message.clone(),
    );

    start_exit_task(sender_exit).await;

    mqtt_loop_handle
        .await
        .expect("Error while waiting for tasks to shut down");

    Ok(())
}

fn start_output_task(
    mut receiver: Receiver<MessageEvent>,
    topic_storage: Arc<TopicStorage>,
    sender_message: Sender<MessageEvent>,
) {
    task::spawn(async move {
        loop {
            if let Ok(MessageEvent::ReceivedFiltered(message)) = receiver.recv().await {
                let outputs = topic_storage.get_outputs_for_topic(&message.topic);
                for output in outputs {
                    if let Err(e) = write_to_output(sender_message.clone(), &message, output) {
                        error!("Error while writing to output {}: {e:?}", output.target);
                    }
                }
            }
        }
    });
}

fn write_to_output(
    sender_message: Sender<MessageEvent>,
    message: &MessageReceivedData,
    output: &Output,
) -> Result<(), OutputError> {
    let conv = PayloadFormat::try_from((message.payload.clone(), output.format()))?;
    match output.target() {
        OutputTarget::Console(_options) => ConsoleOutput::output(
            &message.topic,
            conv.clone().try_into()?,
            conv,
            message.qos,
            message.retain,
        ),
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
    }
}

fn start_sparkplug_monitor(
    sparkplug_network: Arc<Mutex<SparkplugNetwork>>,
    mut receiver: Receiver<MessageEvent>,
) {
    tracing::debug!("Starting sparkplug network monitor");

    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(MessageEvent::ReceivedUnfiltered(message)) => {
                    if let PayloadFormat::Sparkplug(payload) = message.payload {
                        tracing::debug!("Received sparkplug message on topic {}", message.topic);
                        tracing::trace!("{}", payload);
                        if let Err(e) = sparkplug_network
                            .lock()
                            .await
                            .try_parse_message(message.topic, payload)
                        {
                            tracing::error!("Error while parsing sparkplug message: {e:?}");
                        }
                    }
                }
                Err(RecvError::Lagged(skipped_messages)) => {
                    tracing::warn!("Receiver skipped {skipped_messages} messages");
                }
                Err(RecvError::Closed) => break,
                _ => {}
            }
        }

        tracing::debug!("Sparkplug network monitor exited");
    });
}

fn start_scheduler_monitor_task(
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

fn start_scheduler_task(
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

fn start_subscription_task(
    mqtt_service: Arc<Mutex<dyn MqttService>>,
    sender: Sender<MqttReceiveEvent>,
    topics: Vec<(Subscription, String)>,
) {
    let mut receiver_connect = sender.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver_connect.recv().await {
            match event {
                MqttReceiveEvent::V5(rumqttc::v5::Event::Incoming(Incoming::ConnAck(_)))
                | MqttReceiveEvent::V311(rumqttc::Event::Incoming(IncomingV311::ConnAck(_))) => {
                    for (subscription, topic) in topics.iter() {
                        info!(
                            "Subscribing to topic {} with QoS {:?}",
                            topic,
                            subscription.qos()
                        );
                        if let Err(e) = mqtt_service
                            .lock()
                            .await
                            .subscribe(topic.clone(), *subscription.qos())
                            .await
                        {
                            error!("Could not subscribe to topic {}: {}", topic, e);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}

fn start_publish_task(
    mut receiver_publish: Receiver<MessageEvent>,
    mqtt_service_publish: Arc<Mutex<dyn MqttService>>,
) {
    tokio::spawn(async move {
        while let Ok(MessageEvent::Publish(event)) = receiver_publish.recv().await {
            mqtt_service_publish.lock().await.publish(event).await;
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

async fn start_exit_task(sender: Sender<()>) {
    task::spawn(async move {
        if let Err(_e) = signal::ctrl_c().await {
            error!("Could not add ctrl + c handler");
        }

        info!("Exit signal received, shutting down");

        if let Err(e) = sender.send(()) {
            warn!("No active listeners for exit signal present: {e:?}");
        };
    });
}

fn init_logger(level: Level) -> Result<(), TryInitError> {
    let subscriber = SubscriberBuilder::default().with_max_level(level).finish();
    subscriber.try_init()
}

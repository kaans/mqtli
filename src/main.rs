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
mod tasks;

use std::sync::Arc;

use crate::args::load_config;
use anyhow::Context;
use mqtlib::config::mqtli_config::{Mode, MqttVersion};
use mqtlib::config::subscription::Subscription;
use mqtlib::config::PayloadType;
use mqtlib::mqtt::mqtt_handler::MqttHandler;
use mqtlib::mqtt::v311::mqtt_service::MqttServiceV311;
use mqtlib::mqtt::v5::mqtt_service::MqttServiceV5;
use mqtlib::mqtt::{MessageEvent, MqttReceiveEvent, MqttService};
use mqtlib::publish::trigger_periodic::TriggerPeriodic;
use mqtlib::sparkplug::network::SparkplugNetwork;
use mqtlib::storage::get_sql_storage;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};
use tokio::{signal, task};
use tracing::{error, info, trace, warn, Level};
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

    trace!("{}", config);

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

    let mqtt_loop_handle = mqtt_service
        .lock()
        .await
        .connect(sender_receive.clone(), sender_exit.subscribe())
        .await
        .with_context(|| "Error while connecting to mqtt broker")?;

    tasks::publish::start_publish_task(sender_message.subscribe(), mqtt_service.clone());

    let scheduler = TriggerPeriodic::new(mqtt_service.clone()).await;

    tasks::scheduler::start_scheduler_monitor_task(
        mqtt_service.clone(),
        scheduler.get_receiver_command(),
        filtered_subscriptions.clone(),
    );

    tasks::scheduler::start_scheduler_task(
        scheduler,
        sender_receive.clone(),
        topic_storage.clone(),
        sender_exit.subscribe(),
    );

    let mut incoming_messages_handler = MqttHandler::new(topic_storage.clone());
    incoming_messages_handler.start_task(sender_receive.subscribe(), sender_message.clone());

    tasks::subscription::start_subscription_task(
        mqtt_service,
        sender_receive,
        filtered_subscriptions,
    );

    let exclude_types = match config.mode {
        Mode::Sparkplug => vec![PayloadType::Sparkplug],
        _ => vec![],
    };

    let sparkplug_network = Arc::new(Mutex::new(SparkplugNetwork::default()));
    tasks::sparkplug::start_sparkplug_monitor(
        sparkplug_network,
        topic_storage.clone(),
        sender_message.subscribe(),
    );

    let db = if let Some(sql) = &config.sql_storage {
        Some(get_sql_storage(sql).await?)
    } else {
        None
    };

    tasks::output::start_output_task(
        sender_message.subscribe(),
        topic_storage.clone(),
        sender_message,
        exclude_types,
        Arc::new(db),
    );

    start_exit_task(sender_exit).await;

    mqtt_loop_handle
        .await
        .expect("Error while waiting for tasks to shut down");

    Ok(())
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

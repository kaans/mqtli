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
use futures::StreamExt;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::Context;
use log::{debug, error, info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tokio::sync::{broadcast, Mutex};
use tokio::{signal, task};

use crate::config::mqtli_config::{parse_config, MqttVersion};
use crate::config::topic::Topic;
use crate::mqtt::mqtt_handler::MqttHandler;
use crate::mqtt::v311::mqtt_service::MqttServiceV311;
use crate::mqtt::v5::mqtt_service::MqttServiceV5;
use crate::mqtt::{MqttPublishEvent, MqttReceiveEvent, MqttService};
use crate::publish::trigger_periodic::TriggerPeriodic;
use mqtlib::built_info::PKG_VERSION;
use crate::config::publish::PublishTriggerType::Periodic;
use crate::config::subscription::Subscription;

mod config;
mod mqtt;
mod output;
mod payload;
mod publish;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = parse_config().with_context(|| "Error while parsing configuration")?;

    init_logger(config.log_level());

    info!("MQTli version {} starting", PKG_VERSION);

    debug!("{}", config);

    let mqtt_service: Arc<Mutex<dyn MqttService>> = match config.broker().mqtt_version() {
        MqttVersion::V311 => Arc::new(Mutex::new(MqttServiceV311::new(Arc::new(
            config.broker().clone(),
        )))),
        MqttVersion::V5 => Arc::new(Mutex::new(MqttServiceV5::new(Arc::new(
            config.broker().clone(),
        )))),
    };

    let filtered_subscriptions: Vec<(&Subscription, &String)> = config.topics().iter()
        .filter_map(|topic| topic.subscription().as_ref().map(|s| (s, topic.topic())))
        .filter(|(s,_)| *s.enabled()).collect();
    
    futures::stream::iter(filtered_subscriptions)
        .for_each(|(subscription, topic)| async {
            mqtt_service
                .lock()
                .await
                .subscribe(topic.to_string(), *subscription.qos())
                .await;
        }).await;

    let (sender, receiver) = broadcast::channel::<MqttReceiveEvent>(32);
    let (sender_publish, mut receiver_publish) = broadcast::channel::<MqttPublishEvent>(32);

    let topics = Arc::new(config.topics);
    let mut handler = MqttHandler::new(topics.clone());
    handler.start_task(receiver, sender_publish);

    let task_handle_service = mqtt_service
        .lock()
        .await
        .connect(Some(sender))
        .await
        .with_context(|| "Error while connecting to mqtt broker")?;

    start_scheduler(topics.clone(), mqtt_service.clone()).await;

    start_exit_task(mqtt_service.clone()).await;

    tokio::spawn(async move {
        while let Ok(event) = receiver_publish.recv().await {
            mqtt_service.lock().await.publish(event).await;
        }
    });

    task_handle_service
        .await
        .expect("Error while waiting for tasks to shut down");
    handler.await_task().await;

    Ok(())
}

async fn start_scheduler(topics: Arc<Vec<Topic>>, mqtt_service: Arc<Mutex<dyn MqttService>>) {
    let mut scheduler = TriggerPeriodic::new(mqtt_service).await;

    for topic in topics.deref() {
        if let Some(publish) = topic
            .publish()
            .as_ref()
            .filter(|publish| *publish.enabled())
        {
            for trigger in publish.trigger() {
                #[allow(irrefutable_let_patterns)]
                if let Periodic(value) = trigger {
                    if let Err(e) = scheduler
                        .add_schedule(
                            value.interval(),
                            value.count(),
                            value.initial_delay(),
                            topic,
                            publish.qos(),
                            *publish.retain(),
                            publish.input(),
                        )
                        .await
                    {
                        error!("Error while adding schedule: {}", e);
                    };
                }
            }
        }
    }

    let _ = scheduler.start().await;
}

async fn start_exit_task(client: Arc<Mutex<dyn MqttService>>) {
    task::spawn(async move {
        if let Err(_e) = signal::ctrl_c().await {
            error!("Could not add ctrl + c handler");
        }

        info!("Exit signal received, shutting down");

        match client.lock().await.disconnect().await {
            Ok(_) => {
                info!("Successfully disconnected");
            }
            Err(e) => {
                error!("Error during disconnect: {}", e);
            }
        };
    });
}

fn init_logger(filter: &LevelFilter) {
    if TermLogger::init(
        *filter,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .is_err()
    {
        panic!("Another logger was already configured, exiting")
    }
}

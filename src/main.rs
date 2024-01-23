use std::ops::Deref;
use std::sync::Arc;

use anyhow::Context;
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tokio::sync::{broadcast, Mutex};
use tokio::{signal, task};

use crate::config::mqtli_config::PublishTriggerType::Periodic;
use crate::config::mqtli_config::{parse_config, MqttVersion, Topic};
use crate::mqtt::mqtt_handler::MqttHandler;
use crate::mqtt::v311::mqtt_service::MqttServiceV311;
use crate::mqtt::v5::mqtt_service::MqttServiceV5;
use crate::mqtt::{MqttEvent, MqttService};
use crate::publish::trigger_periodic::TriggerPeriodic;

mod config;
mod mqtt;
mod output;
mod payload;
mod publish;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = parse_config().with_context(|| "Error while parsing configuration")?;

    init_logger(config.log_level());

    let mqtt_service: Arc<Mutex<dyn MqttService>> = match config.broker().mqtt_version() {
        MqttVersion::V311 => Arc::new(Mutex::new(MqttServiceV311::new(Arc::new(
            config.broker().clone(),
        )))),
        MqttVersion::V5 => Arc::new(Mutex::new(MqttServiceV5::new(Arc::new(
            config.broker().clone(),
        )))),
    };

    for topic in config.topics() {
        if *topic.subscription().enabled() {
            mqtt_service
                .lock()
                .await
                .subscribe(topic.topic().to_string(), *topic.subscription().qos())
                .await;
        } else {
            info!("Not subscribing to topic, not enabled :{}", topic.topic());
        }
    }

    let (sender, receiver) = broadcast::channel::<MqttEvent>(32);

    let topics = Arc::new(config.topics);
    let mut handler = MqttHandler::new(topics.clone());
    handler.start_task(receiver);

    let task_handle_service = mqtt_service
        .lock()
        .await
        .connect(Some(sender))
        .await
        .with_context(|| "Error while connecting to mqtt broker")?;

    start_scheduler(topics.clone(), mqtt_service.clone()).await;

    start_exit_task(mqtt_service.clone()).await;

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
                if let Periodic(value) = trigger {
                    if let Err(e) = scheduler
                        .add_schedule(
                            value.interval(),
                            value.count(),
                            value.initial_delay(),
                            topic.topic(),
                            publish.qos(),
                            *publish.retain(),
                            publish.input(),
                            topic.payload(),
                        )
                        .await
                    {
                        error!("Error while adding schedule: {:?}", e);
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
                error!("Error during disconnect: {:?}", e);
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

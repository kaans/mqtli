use std::sync::Arc;

use anyhow::Context;
use log::{error, info, LevelFilter};
use simplelog::{Config, SimpleLogger};
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio::{signal, task};

use crate::config::mqtli_config::PublishTriggerType::Periodic;
use crate::config::mqtli_config::{parse_config, Topic};
use crate::mqtt::v5::mqtt_handler::MqttHandlerV5;
use crate::mqtt::v5::mqtt_service::MqttServiceV5;
use crate::publish::trigger_periodic::TriggerPeriodic;

mod config;
mod output;
mod payload;
mod publish;
mod mqtt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = parse_config().with_context(|| "Error while parsing configuration")?;

    init_logger(config.log_level());

    let (sender_exit, receiver_exit) = broadcast::channel(1);

    let mqtt_service = Arc::new(Mutex::new(MqttServiceV5::new(
        Arc::new(config.broker().clone()),
        receiver_exit,
    )));

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

    let (sender, receiver) = broadcast::channel(32);

    let topics = Arc::new(config.topics);
    let mut handler = MqttHandlerV5::new(topics.clone());
    handler.start_task(receiver);

    let task_handle_service = mqtt_service
        .lock()
        .await
        .connect(Some(sender))
        .await
        .with_context(|| "Error while connecting to mqtt broker")?;

    start_scheduler(topics.clone(), mqtt_service.clone()).await;

    start_exit_task(sender_exit).await;
    task_handle_service
        .await
        .expect("Error while waiting for tasks to shut down");
    handler.await_task().await;

    Ok(())
}

async fn start_scheduler(topics: Arc<Vec<Topic>>, mqtt_service: Arc<Mutex<MqttServiceV5>>) {
    let mut scheduler = TriggerPeriodic::new(mqtt_service);

    topics.iter().for_each(|topic| {
        if let Some(publish) = topic
            .publish()
            .as_ref()
            .filter(|publish| *publish.enabled())
        {
            publish.trigger().iter().for_each(|trigger| {
                if let Periodic(value) = trigger {
                    if let Err(e) = scheduler.add_schedule(
                        value.interval(),
                        value.count(),
                        value.initial_delay(),
                        topic.topic(),
                        publish.qos(),
                        *publish.retain(),
                        publish.input(),
                        topic.payload(),
                    ) {
                        error!("Error while adding schedule: {:?}", e);
                    };
                }
            })
        }
    });

    scheduler.start().await;
}

async fn start_exit_task(sender_exit: Sender<i32>) {
    let exit_task: JoinHandle<()> = task::spawn(async move {
        if let Err(_e) = signal::ctrl_c().await {
            error!("Could not add ctrl + c handler");
        }

        info!("Exit signal received, shutting down");

        let _ = sender_exit.send(10);
    });

    exit_task.await.expect("Could not join thread");
}

fn init_logger(filter: &LevelFilter) {
    let config = Config::default();
    if SimpleLogger::init(*filter, config).is_err() {
        panic!("Another logger was already configured, exiting")
    }
}

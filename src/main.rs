use std::process::exit;
use std::sync::Arc;

use anyhow::anyhow;
use log::{error, info, LevelFilter};
use simplelog::{Config, SimpleLogger};
use tokio::{signal, task};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::{parse_config, Topic};
use crate::config::mqtli_config::PublishTriggerType::Periodic;
use crate::mqtt_handler::MqttHandler;
use crate::mqtt_service::MqttService;
use crate::publish::trigger_periodic::TriggerPeriodic;

mod config;
mod mqtt_service;
mod mqtt_handler;
mod payload;
mod output;
mod publish;


#[tokio::main]
async fn main() {
    let config = match parse_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Error while parsing configuration:\n\n{:#}", anyhow!(e));
            exit(1);
        }
    };

    init_logger(config.log_level());

    let (sender_exit, receiver_exit) = broadcast::channel(1);

    let mut mqtt_service = MqttService::new(Arc::new(config.broker().clone()), receiver_exit);

    for topic in config.topics() {
        if *topic.subscription().enabled() {
            mqtt_service.subscribe(topic.topic().to_string(), *topic.subscription().qos()).await;
        } else {
            info!("Not subscribing to topic, not enabled :{}", topic.topic());
        }
    }

    let (sender, receiver) = broadcast::channel(32);

    let topics = Arc::new(Box::new(config.topics));
    let mut handler = MqttHandler::new(topics.clone());
    handler.start_task(receiver);

    if let Err(e) = mqtt_service.connect(Some(sender)).await {
        error!("Error while connecting to mqtt broker: {}", e);
        exit(2);
    }

    start_scheduler(topics.clone()).await;

    start_exit_task(sender_exit).await;
    mqtt_service.await_task().await;
    handler.await_task().await;
}

async fn start_scheduler(topics: Arc<Box<Vec<Topic>>>) {
    let mut scheduler = TriggerPeriodic::new();

    topics.iter().for_each(|topic| for trigger in topic.publish().trigger() {
        if let Periodic(value) = trigger {
            if let Err(e) = scheduler.add_schedule(value.interval(), value.count(), value.initial_delay()) {
                error!("Error while adding schedule: {:?}", e);
            };
        }
    });

    scheduler.start().await;
}

async fn start_exit_task(sender_exit: Sender<i32>) {
    let exit_task: JoinHandle<()> = task::spawn(async move {
        if let Err(_e) = signal::ctrl_c().await {
            error!("Could not add ctrlc handler");
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

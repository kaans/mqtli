use std::process::exit;
use std::sync::Arc;

use anyhow::anyhow;
use log::{error, info, LevelFilter};
use simplelog::{Config, SimpleLogger};
use tokio::{signal, task};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::parse_config;
use crate::mqtt_handler::MqttHandler;
use crate::mqtt_service::MqttService;

mod config;
mod mqtt_service;
mod mqtt_handler;
mod payload;
mod output;


#[tokio::main]
async fn main() {
    let config = match parse_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Error while parsing configuration:\n\n{:#}", anyhow!(e));
            exit(1);
        }
    };

    init_logger(config.logger().level());

    let (sender_exit, receiver_exit) = broadcast::channel(1);

    let mut mqtt_service = MqttService::new(Arc::new(config.broker.clone()), receiver_exit);

    for topic in config.topics() {
        mqtt_service.subscribe((*topic).clone()).await;
    }

    let (sender, receiver) = broadcast::channel(32);

    let mut handler = MqttHandler::new(config.topics());
    handler.start_task(receiver);

    if let Err(e) = mqtt_service.connect(Some(sender)).await {
        error!("Error while connecting to mqtt broker: {}", e);
        exit(2);
    }

    start_exit_task(sender_exit).await;
    mqtt_service.await_task().await;
    handler.await_task().await;
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

use std::process::exit;

use anyhow::anyhow;
use log::{error, LevelFilter};
use simplelog::{Config, SimpleLogger};
use tokio::sync::broadcast;

use crate::config::mqtli_config::parse_config;
use crate::mqtt_handler::MqttHandler;
use crate::mqtt_service::MqttService;

mod config;
mod mqtt_service;
mod mqtt_handler;


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

    let mut mqtt_service = MqttService::new(config.broker());

    for topic in config.subscribe_topics() {
        mqtt_service.subscribe((*topic).clone()).await;
    }

    let (sender, receiver) = broadcast::channel(32);

    let mut handler = MqttHandler::new(config.subscribe_topics());
    handler.start_task(receiver);

    if let Err(e) = mqtt_service.connect(Some(sender)).await {
        error!("Error while connecting to mqtt broker: {}", e);
        exit(2);
    }

    mqtt_service.await_task().await;
    handler.await_task().await;
}

fn init_logger(filter: &LevelFilter) {
    let config = Config::default();
    if SimpleLogger::init(*filter, config).is_err() {
        panic!("Another logger was already configured, exiting")
    }
}

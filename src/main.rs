use std::future::Future;

use clap::Parser;
use log::{debug, error, info, LevelFilter};
use rumqttc::v5::{AsyncClient, MqttOptions};
use rumqttc::v5::mqttbytes::QoS::AtLeastOnce;
use simplelog::{Config, SimpleLogger};

use crate::config::main_config::{MqtliConfig, parse_config};

mod config;


#[tokio::main]
async fn main() {
    let config = parse_config();

    init_logger(config.logger().level());

    info!("MQTli starting");

    let client = start_mqtt(&config).await;

    for topic in config.subscribe_topics() {
        subscribe_to_topic(&client, topic.to_string()).await;
    }

    // wait forever
    std::future::pending::<()>().await;
}


async fn subscribe_to_topic(client: &AsyncClient, topic: String) {
    info!("Subscribing to topic {topic}");

    client.subscribe(topic, AtLeastOnce).await.expect("Could not subscribe to topic {topic}");
}

async fn start_mqtt(config: &MqtliConfig) -> AsyncClient {
    debug!("Connection to {}:{} with client id {}", config.broker().host(),
                config.broker().port(), config.broker().client_id());
    let mut options = MqttOptions::new(config.broker().client_id(),
                                       config.broker().host(),
                                       *config.broker().port());

    debug!("Setting keep alive to {} seconds", config.broker().keep_alive().as_secs());
    options.set_keep_alive(*config.broker().keep_alive());

    let (mut client, mut connection) = AsyncClient::new(options, 10);

    tokio::task::spawn(async move {
        loop {
            match connection.poll().await {
                Ok(value) => {
                    info!("Received {:?}", value);
                }
                Err(e) => {
                    error!("Error while processing mqtt loop: {:?}", e);
                }
            }
        }
    });

    client
}

fn init_logger(filter: &LevelFilter) {
    let config = Config::default();
    if SimpleLogger::init(*filter, config).is_err() {
        panic!("Another logger was already configured, exiting")
    }
}

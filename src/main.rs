use std::fs::read_to_string;
use std::future::Future;
use std::path::PathBuf;

use clap::Parser;
use derive_getters::Getters;
use log::{debug, error, info, LevelFilter};
use rumqttc::v5::{AsyncClient, MqttOptions};
use rumqttc::v5::mqttbytes::QoS::AtLeastOnce;
use simplelog::{Config, SimpleLogger};

use crate::args::{LoggingArgs, MqttBrokerConnectArgs};
use crate::config_file::ConfigFile;

mod args;
mod config_file;

#[derive(Parser, Debug, Getters)]
#[command(author, version, about, long_about = None)]
struct MqtliArgs {
    #[command(flatten)]
    broker: MqttBrokerConnectArgs,

    #[command(flatten)]
    logger: LoggingArgs,

    #[arg(long = "config-file", default_value = "config.yaml", env = "CONFIG_FILE_PATH")]
    config_file: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = MqtliArgs::parse();

    init_logger(args.logger.level());

    info!("MQTli starting");

    let config = read_config(&args.config_file);

    let client = start_mqtt(args).await;

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

fn read_config(buf: &PathBuf) -> ConfigFile {
    let content = read_to_string(buf).expect("Could not read config file");

    let config = serde_yaml::from_str(content.as_str()).expect("Invalid config file");

    config
}

async fn start_mqtt(args: MqtliArgs) -> AsyncClient {
    let mut options = MqttOptions::new(args.broker.client_id(),
                                       args.broker.host(),
                                       *args.broker.port());

    debug!("Setting keep alive to {} seconds", args.broker.keep_alive().as_secs());
    options.set_keep_alive(*args.broker.keep_alive());

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

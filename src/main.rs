use std::future::Future;

use clap::Parser;
use derive_getters::Getters;
use log::{debug, error, info, LevelFilter};
use rumqttc::v5::{AsyncClient, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;
use simplelog::{Config, SimpleLogger};

use crate::args::{LoggingArgs, MqttBrokerConnectArgs};

mod args;

#[derive(Parser, Debug, Getters)]
#[command(author, version, about, long_about = None)]
struct MqtliArgs {
    #[command(flatten)]
    broker: MqttBrokerConnectArgs,

    #[command(flatten)]
    logger: LoggingArgs,
}

#[tokio::main]
async fn main() {
    let args = MqtliArgs::parse();

    init_logger(args.logger.level());

    info!("MQTli starting");

    start_mqtt(args).await;

    // wait forever
    std::future::pending::<()>().await;
}

async fn start_mqtt(args: MqtliArgs) {
    let mut options = MqttOptions::new(args.broker.client_id(),
                                       args.broker.host(),
                                       *args.broker.port());

    debug!("Setting keep alive to {} seconds", args.broker.keep_alive().as_secs());
    options.set_keep_alive(*args.broker.keep_alive());

    let (mut client, mut connection) = AsyncClient::new(options, 10);

    client.subscribe("mqtcli/test", QoS::AtLeastOnce).await.expect("Could not subscribe");

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
}

fn init_logger(filter: &LevelFilter) {
    let config = Config::default();
    if SimpleLogger::init(*filter, config).is_err() {
        panic!("Another logger was already configured, exiting")
    }
}

use clap::Parser;
use derive_getters::Getters;
use log::{info, LevelFilter};
use simplelog::{Config, SimpleLogger};

use crate::args::{LoggingArgs, MqttBrokerConnectArgs};

mod args;

#[derive(Parser, Debug, Getters)]
#[command(author, version, about, long_about = None)]
struct MqtliArgs {

    #[command(flatten)]
    broker: MqttBrokerConnectArgs,

    #[command(flatten)]
    logger: LoggingArgs
}

fn main() {
    let args = MqtliArgs::parse();

    init_logger(args.logger.level());

    info!("MQTli starting");

}

fn init_logger(filter: &LevelFilter) {
    let config = Config::default();
    if SimpleLogger::init(*filter, config).is_err() {
        panic!("Another logger was already configured, exiting")
    }
}

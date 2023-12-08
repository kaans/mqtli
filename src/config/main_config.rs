use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use clap::Parser;
use derive_getters::Getters;
use log::LevelFilter;

use crate::config::args::MqtliArgs;
use crate::config::config_file::read_config;

#[derive(Debug, Default, Getters)]
pub struct MqtliConfig {
    broker: MqttBrokerConnectArgs,

    logger: LoggingArgs,

    config_file: PathBuf,

    subscribe_topics: Vec<String>
}

#[derive(Debug, Default, Getters)]
pub struct MqttBrokerConnectArgs {
    host: String,
    port: u16,
    client_id: String,
    keep_alive: Duration,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Getters)]
pub struct LoggingArgs {
    level: LevelFilter,
}

impl Default for LoggingArgs {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
        }
    }
}

pub fn parse_config() -> MqtliConfig {
    let mut args = MqtliArgs::parse();
    let config_file = read_config(&args.config_file());

    let mut config = MqtliConfig {
        ..Default::default()
    };

    config.broker.host = args.broker().host().clone().or(config_file.host().clone()).or(Some("localhost".to_string())).unwrap();
    config.broker.port = args.broker().port().or(config_file.port().clone()).or(Some(1883)).unwrap();
    config.broker.client_id = args.broker().client_id().clone().or(config_file.client_id().clone()).or(Some("mqtli".to_string())).unwrap();
    config.broker.keep_alive = args.broker().keep_alive().or(config_file.keep_alive().clone()).or(Some(Duration::from_secs(5))).unwrap();
    config.broker.username = args.broker().username().clone().or(config_file.username().clone()).or(None);
    config.broker.password = args.broker().password().clone().or(config_file.password().clone()).or(None);

    config.logger.level = args.logger().level().or(config_file.log_level().clone()
        .map(|v| LevelFilter::from_str(v.as_str()).expect("Invalid log level {v}")))
        .or(Option::from(LevelFilter::Info)).unwrap();

    for topic in config_file.subscribe_topics() {
        config.subscribe_topics.push(topic.clone());
    }

    config
}

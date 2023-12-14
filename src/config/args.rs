use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser};
use derive_getters::Getters;
use log::LevelFilter;
use rumqttc::v5::mqttbytes::QoS;

use crate::config::mqtli_config::TlsVersion;

#[derive(Parser, Debug, Getters)]
#[command(author, version, about, long_about = None)]
pub struct MqtliArgs {
    #[command(flatten)]
    broker: MqttBrokerConnectArgs,

    #[command(flatten)]
    logger: LoggingArgs,

    #[arg(long = "config-file", default_value = "config.yaml", env = "CONFIG_FILE_PATH")]
    config_file: PathBuf,

    topics: Vec<String>
}

#[derive(Args, Debug, Default, Getters)]
#[group(required = false, multiple = true)]
pub struct MqttBrokerConnectArgs {
    #[arg(short = 'o', long = "host", env = "BROKER_HOST")]
    host: Option<String>,

    #[arg(short = 'p', long = "port", env = "BROKER_PORT")]
    port: Option<u16>,

    #[arg(short = 'c', long = "client-id",  env = "BROKER_CLIENT_ID")]
    client_id: Option<String>,

    #[arg(long = "keep-alive", env = "BROKER_KEEP_ALIVE", value_parser = parse_keep_alive)]
    keep_alive: Option<Duration>,

    #[arg(short = 'u', long = "username", env = "BROKER_USERNAME")]
    username: Option<String>,

    #[arg(short = 'w', long = "password", env = "BROKER_PASSWORD")]
    password: Option<String>,

    #[arg(long = "use-tls", env = "BROKER_USE_TLS", help_heading = "TLS")]
    use_tls: Option<bool>,

    #[arg(long = "ca-file", env = "BROKER_TLS_CA_FILE", help_heading = "TLS")]
    tls_ca_file: Option<PathBuf>,

    #[arg(long = "client-cert", env = "BROKER_TLS_CLIENT_CERTIFICATE_FILE", help_heading = "TLS")]
    tls_client_certificate: Option<PathBuf>,

    #[arg(long = "client-key", env = "BROKER_TLS_CLIENT_KEY_FILE", help_heading = "TLS")]
    tls_client_key: Option<PathBuf>,

    #[arg(long = "tls-version", env = "BROKER_TLS_VERSION", value_enum, help_heading = "TLS")]
    tls_version: Option<TlsVersion>,

    #[command(flatten)]
    last_will: Option<LastWillConfig>
}

#[derive(Args, Debug, Default, Getters)]
pub struct LastWillConfig {
    #[arg(long = "last-will-payload", env = "BROKER_LAST_WILL_PAYLOAD", help_heading = "Last will")]
    payload: Option<String>,

    #[arg(long = "last-will-topic", env = "BROKER_LAST_WILL_TOPIC", help_heading = "Last will")]
    topic: Option<String>,

    #[arg(long = "last-will-qos", env = "BROKER_LAST_WILL_QOS", value_parser = parse_qos, help_heading = "Last will", help = "0 = at most once; 1 = at least once; 2 = exactly once")]
    qos: Option<QoS>,

    #[arg(long = "last-will-retain", env = "BROKER_LAST_WILL_RETAIN", help_heading = "Last will")]
    retain: Option<bool>
}

#[derive(Args, Debug, Getters)]
#[group(required = false, multiple = true)]
pub struct LoggingArgs {
    #[arg(short = 'l', long = "log-level", env = "LOG_LEVEL", help_heading = "Logging")]
    level: Option<LevelFilter>,
}

fn parse_keep_alive(input: &str) -> Result<Duration, String> {
    let duration_in_seconds: u64 = input.parse()
        .map_err(|_| format!("{input} is not a valid duration in seconds"))?;

    Ok(Duration::from_secs(duration_in_seconds))
}

fn parse_qos(input: &str) -> Result<QoS, String> {
    let qos: QoS = match input {
        "0" => QoS::AtMostOnce,
        "1" => QoS::AtLeastOnce,
        "2" => QoS::ExactlyOnce,
        _ => return Err("QoS value must be 0, 1 or 2".to_string())
    };

    Ok(qos)
}
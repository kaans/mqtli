use std::fmt::Debug;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use clap::{Args, Parser};
use derive_getters::Getters;
use log::LevelFilter;
use rumqttc::v5::mqttbytes::QoS;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Unexpected};

use crate::config::{args, ConfigError, OutputFormat};
use crate::config::mqtli_config::TlsVersion;

#[derive(Debug, Deserialize, Parser)]
#[command(author, version, about, long_about = None)]
pub struct MqtliArgs {
    #[command(flatten)]
    pub broker: MqttBrokerConnectArgs,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_level_filter")]
    #[arg(short = 'l', long = "log-level", env = "LOG_LEVEL", help_heading = "Logging")]
    pub log_level: Option<LevelFilter>,

    #[arg(long = "config-file", env = "CONFIG_FILE_PATH")]
    #[serde(skip_serializing)]
    pub config_file: Option<PathBuf>,

    #[clap(skip)]
    #[serde(default)]
    pub topics: Vec<Topic>,
}

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct MqttBrokerConnectArgs {
    #[arg(short = 'o', long = "host", env = "BROKER_HOST")]
    pub host: Option<String>,

    #[arg(short = 'p', long = "port", env = "BROKER_PORT")]
    pub port: Option<u16>,

    #[arg(short = 'c', long = "client-id", env = "BROKER_CLIENT_ID")]
    pub client_id: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    #[arg(long = "keep-alive", env = "BROKER_KEEP_ALIVE", value_parser = parse_keep_alive)]
    pub keep_alive: Option<Duration>,

    #[arg(short = 'u', long = "username", env = "BROKER_USERNAME")]
    pub username: Option<String>,

    #[arg(short = 'w', long = "password", env = "BROKER_PASSWORD")]
    pub password: Option<String>,

    #[arg(long = "use-tls", env = "BROKER_USE_TLS", help_heading = "TLS")]
    pub use_tls: Option<bool>,

    #[arg(long = "ca-file", env = "BROKER_TLS_CA_FILE", help_heading = "TLS")]
    pub tls_ca_file: Option<PathBuf>,

    #[arg(long = "client-cert", env = "BROKER_TLS_CLIENT_CERTIFICATE_FILE", help_heading = "TLS")]
    pub tls_client_certificate: Option<PathBuf>,

    #[arg(long = "client-key", env = "BROKER_TLS_CLIENT_KEY_FILE", help_heading = "TLS")]
    pub tls_client_key: Option<PathBuf>,

    #[arg(long = "tls-version", env = "BROKER_TLS_VERSION", value_enum, help_heading = "TLS")]
    pub tls_version: Option<TlsVersion>,

    #[command(flatten)]
    pub last_will: Option<LastWillConfig>,
}

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct LastWillConfig {
    #[arg(long = "last-will-payload", env = "BROKER_LAST_WILL_PAYLOAD", help_heading = "Last will")]
    pub payload: Option<String>,

    #[arg(long = "last-will-topic", env = "BROKER_LAST_WILL_TOPIC", help_heading = "Last will")]
    pub topic: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos_option")]
    #[arg(long = "last-will-qos", env = "BROKER_LAST_WILL_QOS", value_parser = parse_qos, help_heading = "Last will", help = "0 = at most once; 1 = at least once; 2 = exactly once")]
    pub qos: Option<QoS>,

    #[arg(long = "last-will-retain", env = "BROKER_LAST_WILL_RETAIN", help_heading = "Last will")]
    pub retain: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Getters)]
pub struct Topic {
    pub topic: String,
    pub subscription: Option<Subscription>,
    pub payload: Option<PayloadType>,
    pub publish: Option<Publish>,
}

#[derive(Debug, Default, Deserialize, Getters)]
pub struct Publish {
    #[serde(default)]
    enabled: bool,

    #[serde(default)]
    retain: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
    trigger: Option<Vec<PublishTriggerType>>,

    input: PublishInputType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PublishInputType {
    #[serde(rename = "text")]
    Text(PublishInputTypeContentPath),
    #[serde(rename = "raw")]
    Raw(PublishInputTypePath),
}

impl Default for PublishInputType {
    fn default() -> Self {
        Self::Text(PublishInputTypeContentPath::default())
    }
}

#[derive(Debug, Default, Deserialize, Getters)]
pub struct PublishInputTypeContentPath {
    content: Option<String>,
    path: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize, Getters)]
pub struct PublishInputTypePath {
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PublishTriggerType {
    #[serde(rename = "periodic")]
    Periodic(PublishTriggerTypePeriodic)
}

#[derive(Debug, Default, Deserialize, Getters)]
pub struct PublishTriggerTypePeriodic {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_milliseconds")]
    interval: Option<Duration>,
    count: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_milliseconds")]
    initial_delay: Option<Duration>,
}

#[derive(Debug, Default, Deserialize, Getters, PartialEq)]
pub struct Output {
    format: Option<OutputFormat>,
    target: Option<OutputTarget>,
}


#[derive(Debug, Default, Deserialize, Getters, PartialEq)]
pub struct OutputTargetConsole {}

#[derive(Debug, Deserialize, Getters, PartialEq)]
pub struct OutputTargetFile {
    path: PathBuf,

    #[serde(default)]
    overwrite: bool,
    prepend: Option<String>,
    append: Option<String>,
}

impl Default for OutputTargetFile {
    fn default() -> Self {
        OutputTargetFile {
            path: Default::default(),
            overwrite: false,
            prepend: None,
            append: Some("\n".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PayloadType {
    #[serde(rename = "text")]
    Text(PayloadText),
    #[serde(rename = "protobuf")]
    Protobuf(PayloadProtobuf),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OutputTarget {
    #[serde(rename = "console")]
    Console(OutputTargetConsole),
    #[serde(rename = "file")]
    File(OutputTargetFile),
}

#[derive(Debug, Default, Deserialize, Getters, PartialEq)]
pub struct Subscription {
    enabled: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
    outputs: Option<Vec<Output>>,
}

#[derive(Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadText {}

#[derive(Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadProtobuf {
    definition: PathBuf,
    message: String,
}

pub fn read_config(buf: &PathBuf) -> Result<MqtliArgs, ConfigError> {
    let content = match read_to_string(buf) {
        Ok(content) => content,
        Err(e) => {
            return Err(ConfigError::CouldNotReadConfigFile(e, PathBuf::from(buf)));
        }
    };

    let config: MqtliArgs = match serde_yaml::from_str(content.as_str()) {
        Ok(config) => config,
        Err(e) => {
            return Err(ConfigError::CouldNotParseConfigFile(e, PathBuf::from(buf)));
        }
    };

    Ok(config)
}

pub fn read_cli_args() -> MqtliArgs {
    args::MqtliArgs::parse()
}

fn deserialize_duration_seconds<'a, D>(deserializer: D) -> Result<Option<Duration>, D::Error> where D: Deserializer<'a> {
    let value: u64 = Deserialize::deserialize(deserializer)?;
    return Ok(Some(Duration::from_secs(value)));
}

fn deserialize_duration_milliseconds<'a, D>(deserializer: D) -> Result<Option<Duration>, D::Error> where D: Deserializer<'a> {
    let value: u64 = Deserialize::deserialize(deserializer)?;
    return Ok(Some(Duration::from_millis(value)));
}

fn deserialize_qos<'a, D>(deserializer: D) -> Result<QoS, D::Error> where D: Deserializer<'a> {
    let value: &str = Deserialize::deserialize(deserializer)?;

    if let Ok(int_value) = value.parse::<u8>() {
        return Ok(match int_value {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        });
    }

    Err(Error::invalid_value(Unexpected::Other(value), &"unsigned integer between 0 and 2"))
}

fn deserialize_qos_option<'a, D>(deserializer: D) -> Result<Option<QoS>, D::Error> where D: Deserializer<'a> {
    Ok(Some(deserialize_qos(deserializer)?))
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

fn deserialize_level_filter<'a, D>(deserializer: D) -> Result<Option<LevelFilter>, D::Error> where D: Deserializer<'a> {
    let value: &str = Deserialize::deserialize(deserializer)?;

    let level = match LevelFilter::from_str(value) {
        Ok(level) => level,
        Err(_) => return Err(Error::invalid_value(Unexpected::Other(value), &"unsigned integer between 0 and 2"))
    };

    Ok(Some(level))
}

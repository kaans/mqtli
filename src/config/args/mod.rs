use std::fmt::Debug;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use clap::{Args, Parser};
use derive_getters::Getters;
use log::LevelFilter;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};

use crate::config::filter::FilterType;
use crate::config::mqtli_config::{MqttProtocol, MqttVersion, TlsVersion};
use crate::config::{args, ConfigError, PayloadType, PublishInputType};
use crate::mqtt::QoS;

#[derive(Debug, Deserialize, Parser)]
#[command(author, version, about, long_about = None)]
#[clap(disable_version_flag = true)]
#[clap(disable_help_flag = true)]
pub struct MqtliArgs {
    #[clap(long, action = clap::ArgAction::HelpLong, help = "Print help")]
    help: Option<bool>,

    #[clap(long, action = clap::ArgAction::Version, help = "Print version")]
    version: Option<bool>,

    #[command(flatten)]
    pub broker: Option<MqttBrokerConnectArgs>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_level_filter")]
    #[arg(
        short = 'l',
        long = "log-level",
        env = "LOG_LEVEL",
        help_heading = "Logging",
        help = "Log level (default: info) (possible values: trace, debug, info, warn, error, off)"
    )]
    pub log_level: Option<LevelFilter>,

    #[arg(
        short = 'c',
        long = "config-file",
        env = "CONFIG_FILE_PATH",
        help = "Path to the config file (default: config.yaml)"
    )]
    #[serde(skip_serializing)]
    pub config_file: Option<PathBuf>,

    #[clap(skip)]
    #[serde(default)]
    pub topics: Vec<Topic>,
}

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct MqttBrokerConnectArgs {
    #[arg(
        short = 'h',
        long = "host",
        env = "BROKER_HOST",
        help_heading = "Broker",
        help = "The ip address or hostname of the broker (default: localhost)"
    )]
    pub host: Option<String>,

    #[arg(
        short = 'p',
        long = "port",
        env = "BROKER_PORT",
        help_heading = "Broker",
        help = "The port the broker is listening on (default: 1883)"
    )]
    pub port: Option<u16>,

    #[arg(
        long = "protocol",
        env = "BROKER_PROTOCOL",
        help_heading = "Broker",
        help = "The protocol to use to communicate with the broker (default: tcp)"
    )]
    pub protocol: Option<MqttProtocol>,

    #[arg(
        short = 'i',
        long = "client-id",
        env = "BROKER_CLIENT_ID",
        help_heading = "Broker",
        help = "The client id for this mqtli instance (default: mqtli)"
    )]
    pub client_id: Option<String>,

    #[arg(
        short = 'v',
        long = "mqtt-version",
        env = "BROKER_MQTT_VERSION",
        help_heading = "Broker",
        help = "The MQTT version to use (default: v5)"
    )]
    pub mqtt_version: Option<MqttVersion>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    #[arg(long = "keep-alive", env = "BROKER_KEEP_ALIVE", value_parser = parse_keep_alive, help_heading = "Broker", help = "Keep alive time in seconds (default: 5 seconds)")]
    pub keep_alive: Option<Duration>,

    #[arg(
        short = 'u',
        long = "username",
        env = "BROKER_USERNAME",
        help_heading = "Broker",
        help = "(optional) Username used to authenticate against the broker; if used then username must be given too (default: empty)"
    )]
    pub username: Option<String>,

    #[arg(
        short = 'w',
        long = "password",
        env = "BROKER_PASSWORD",
        help_heading = "Broker",
        help = "(optional) Password used to authenticate against the broker; if used then password must be given too (default: empty)"
    )]
    pub password: Option<String>,

    #[arg(
        long = "use-tls",
        env = "BROKER_USE_TLS",
        help_heading = "TLS",
        help = "If specified, TLS is used to communicate with the broker (default: false)"
    )]
    pub use_tls: Option<bool>,

    #[arg(
        long = "ca-file",
        env = "BROKER_TLS_CA_FILE",
        help_heading = "TLS",
        help = "Path to a PEM encoded ca certificate to verify the broker's certificate (default: empty)"
    )]
    pub tls_ca_file: Option<PathBuf>,

    #[arg(
        long = "client-cert",
        env = "BROKER_TLS_CLIENT_CERTIFICATE_FILE",
        help_heading = "TLS",
        help = "(optional) Path to a PEM encoded client certificate for authenticating against the broker; must be specified with client-key (default: empty)"
    )]
    pub tls_client_certificate: Option<PathBuf>,

    #[arg(
        long = "client-key",
        env = "BROKER_TLS_CLIENT_KEY_FILE",
        help_heading = "TLS",
        help = "(optional) Path to a PKCS#8 encoded, unencrypted client private key for authenticating against the broker; must be specified with client-cert (default: empty)"
    )]
    pub tls_client_key: Option<PathBuf>,

    #[arg(
        long = "tls-version",
        env = "BROKER_TLS_VERSION",
        help_heading = "TLS",
        help = "TLS version to used (default: all)"
    )]
    pub tls_version: Option<TlsVersion>,

    #[command(flatten)]
    pub last_will: Option<LastWillConfig>,
}

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct LastWillConfig {
    #[arg(
        long = "last-will-payload",
        env = "BROKER_LAST_WILL_PAYLOAD",
        help_heading = "Last will",
        help = "The UTF-8 encoded payload of the will message (default: empty)"
    )]
    pub payload: Option<String>,

    #[arg(
        long = "last-will-topic",
        env = "BROKER_LAST_WILL_TOPIC",
        help_heading = "Last will",
        help = "The topic where the last will message will be published (default: empty)"
    )]
    pub topic: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos_option")]
    #[arg(long = "last-will-qos", env = "BROKER_LAST_WILL_QOS", value_parser = parse_qos,
    help_heading = "Last will",
    help = "Quality of Service (default: 0) (possible values: 0 = at most once; 1 = at least once; 2 = exactly once)")
    ]
    pub qos: Option<QoS>,

    #[arg(
        long = "last-will-retain",
        env = "BROKER_LAST_WILL_RETAIN",
        help_heading = "Last will",
        help = "If true, last will message will be retained, else not (default: false)"
    )]
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
pub enum PublishTriggerType {
    #[serde(rename = "periodic")]
    Periodic(PublishTriggerTypePeriodic),
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
    format: Option<PayloadType>,
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

#[derive(Debug, Default, Deserialize, Getters, PartialEq)]
pub struct OutputTargetTopic {
    topic: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
    #[serde(default)]
    retain: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OutputTarget {
    #[serde(rename = "console")]
    Console(OutputTargetConsole),
    #[serde(rename = "file")]
    File(OutputTargetFile),
    #[serde(rename = "topic")]
    Topic(OutputTargetTopic),
}

#[derive(Debug, Default, Deserialize, Getters, PartialEq)]
pub struct Subscription {
    enabled: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
    outputs: Option<Vec<Output>>,
    filters: Option<Vec<FilterType>>,
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

fn deserialize_duration_seconds<'a, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'a>,
{
    let value: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Some(Duration::from_secs(value)))
}

fn deserialize_duration_milliseconds<'a, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'a>,
{
    let value: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Some(Duration::from_millis(value)))
}

fn deserialize_qos<'a, D>(deserializer: D) -> Result<QoS, D::Error>
where
    D: Deserializer<'a>,
{
    let value: Result<u8, _> = Deserialize::deserialize(deserializer);

    if let Ok(int_value) = value {
        return Ok(match int_value {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        });
    }

    Err(Error::invalid_value(
        Unexpected::Other("unknown"),
        &"unsigned integer between 0 and 2",
    ))
}

fn deserialize_qos_option<'a, D>(deserializer: D) -> Result<Option<QoS>, D::Error>
where
    D: Deserializer<'a>,
{
    Ok(Some(deserialize_qos(deserializer)?))
}

fn parse_keep_alive(input: &str) -> Result<Duration, String> {
    let duration_in_seconds: u64 = input
        .parse()
        .map_err(|_| format!("{input} is not a valid duration in seconds"))?;

    Ok(Duration::from_secs(duration_in_seconds))
}

fn parse_qos(input: &str) -> Result<QoS, String> {
    let qos: QoS = match input {
        "0" => QoS::AtMostOnce,
        "1" => QoS::AtLeastOnce,
        "2" => QoS::ExactlyOnce,
        _ => return Err("QoS value must be 0, 1 or 2".to_string()),
    };

    Ok(qos)
}

fn deserialize_level_filter<'a, D>(deserializer: D) -> Result<Option<LevelFilter>, D::Error>
where
    D: Deserializer<'a>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;

    let level = match LevelFilter::from_str(value) {
        Ok(level) => level,
        Err(_) => {
            return Err(Error::invalid_value(
                Unexpected::Other(value),
                &"unsigned integer between 0 and 2",
            ));
        }
    };

    Ok(Some(level))
}

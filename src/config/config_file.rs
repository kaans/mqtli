use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Duration;
use derive_getters::Getters;
use rumqttc::v5::mqttbytes::QoS;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Unexpected};
use crate::config::ConfigError;

#[derive(Debug, PartialEq, Serialize, Deserialize, Getters)]
pub struct ConfigFile {
    host: Option<String>,
    port: Option<u16>,
    client_id: Option<String>,

    #[serde(default)]
    #[serde(serialize_with = "serialize_keep_alive")]
    #[serde(deserialize_with = "deserialize_keep_alive")]
    keep_alive: Option<Duration>,

    username: Option<String>,
    password: Option<String>,

    use_tls: Option<bool>,
    tls_ca_file: Option<PathBuf>,
    tls_client_certificate: Option<PathBuf>,
    tls_client_key: Option<PathBuf>,

    log_level: Option<String>,

    topics: Vec<Topic>
}

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq)]
pub struct Topic {
    topic: String,
    subscription: Option<Subscription>,
    payload: Option<PayloadType>,
    outputs: Option<Vec<Output>>
}

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq)]
pub struct Output {
    format: Option<OutputFormat>,
    target: Option<OutputTarget>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "yaml")]
    Yaml,
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "base64")]
    Base64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OutputTarget {
    #[serde(rename = "console")]
    Console(OutputTargetConsole),
    #[serde(rename = "file")]
    File(OutputTargetFile),
}

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq)]
pub struct OutputTargetConsole {
}

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq)]
pub struct OutputTargetFile {
    path: PathBuf,

    #[serde(default)]
    overwrite: bool,
    prepend: Option<String>,
    append: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq)]
pub struct Subscription {
    enabled: bool,

    #[serde(default)]
    #[serde(serialize_with = "serialize_qos")]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PayloadType {
    #[serde(rename = "text")]
    Text(PayloadText),
    #[serde(rename = "protobuf")]
    Protobuf(PayloadProtobuf)
}

#[derive(Debug, Default, Serialize, Deserialize, Getters, PartialEq)]
pub struct PayloadText {}

#[derive(Debug, Default, Serialize, Deserialize, Getters, PartialEq)]
pub struct PayloadProtobuf {
    definition: PathBuf,
    message: String
}

pub fn read_config(buf: &PathBuf) -> Result<ConfigFile, ConfigError> {
    let content = match read_to_string(buf)  {
        Ok(content) => content,
        Err(e) => {
            return Err(ConfigError::CouldNotReadConfigFile(e, PathBuf::from(buf)));
        }
    };

    let config: ConfigFile = match serde_yaml::from_str(content.as_str()) {
        Ok(config) => config,
        Err(e) => {
            return Err(ConfigError::CouldNotParseConfigFile(e, PathBuf::from(buf)));
        }
    };

    Ok(config)
}

fn serialize_keep_alive<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_u64(value.unwrap().as_secs())
}

fn deserialize_keep_alive<'a, D>(deserializer: D) -> Result<Option<Duration>, D::Error> where D: Deserializer<'a> {
    let value: &str = Deserialize::deserialize(deserializer)?;

    if let Ok(value) = value.parse() {
        return Ok(Some(Duration::from_secs(value)))
    }

    Err(Error::invalid_value(Unexpected::Other(value), &"unsigned integer between 0 and 65535"))
}

fn serialize_qos<S>(value: &QoS, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let int_value = match value {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce => 1,
        QoS::ExactlyOnce => 2
    };

    serializer.serialize_u8(int_value)
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

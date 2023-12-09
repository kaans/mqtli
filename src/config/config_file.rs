use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Duration;
use derive_getters::Getters;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Unexpected};

#[derive(Debug, PartialEq, Serialize, Deserialize, Getters)]
pub struct ConfigFile {
    host: Option<String>,
    port: Option<u16>,
    client_id: Option<String>,
    #[serde(serialize_with = "serialize_keep_alive")]
    #[serde(deserialize_with = "deserialize_keep_alive")]
    keep_alive: Option<Duration>,
    username: Option<String>,
    password: Option<String>,

    log_level: Option<String>,

    subscribe_topics: Vec<String>
}

pub fn read_config(buf: &PathBuf) -> ConfigFile {
    let content = read_to_string(buf).expect("Could not read config file");

    let config = serde_yaml::from_str(content.as_str()).expect("Invalid config file");

    config
}

fn serialize_keep_alive<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    println!("{:?}", value);

    serializer.serialize_bool(true)
}

fn deserialize_keep_alive<'a, D>(deserializer: D) -> Result<Option<Duration>, D::Error> where D: Deserializer<'a> {
    let value: &str = Deserialize::deserialize(deserializer)?;

    if let Ok(value) = value.parse() {
        return Ok(Some(Duration::from_secs(value)))
    }

    Err(D::Error::invalid_value(Unexpected::Other(value), &"unsigned integer between 0 and 65535"))
}
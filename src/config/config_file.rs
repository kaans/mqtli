use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Duration;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Getters)]
pub struct ConfigFile {
    host: Option<String>,
    port: Option<u16>,
    client_id: Option<String>,
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

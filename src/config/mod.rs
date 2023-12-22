use std::io;
use std::path::PathBuf;
use serde::Deserialize;

use thiserror::Error;
use validator::ValidationErrors;

pub mod mqtli_config;
mod args;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file \"{1}\"")]
    CouldNotReadConfigFile(#[source] io::Error, PathBuf),
    #[error("Could not parse config file \"{1}\"")]
    CouldNotParseConfigFile(#[source] serde_yaml::Error, PathBuf),
    #[error("Invalid configuration")]
    InvalidConfiguration(#[source] ValidationErrors),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub enum OutputFormat {
    #[default]
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
    #[serde(rename = "raw")]
    Raw,
}

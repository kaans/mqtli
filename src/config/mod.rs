use std::io;

use thiserror::Error;
use validator::ValidationErrors;

pub mod mqtl_config;
mod config_file;
mod args;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file")]
    CouldNotReadConfigFile(#[source] io::Error),
    #[error("Could not parse config file")]
    CouldNotParseConfigFile(#[source] serde_yaml::Error),
    #[error("Invalid configuration")]
    InvalidConfiguration(#[source] ValidationErrors),
}
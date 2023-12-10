use std::io;
use std::path::PathBuf;

use thiserror::Error;
use validator::ValidationErrors;

pub mod mqtli_config;
mod config_file;
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
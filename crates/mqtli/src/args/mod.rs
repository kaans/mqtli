mod content;
mod parsers;

use std::fmt::Debug;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

use crate::args::content::MqtliArgs;
use clap::Parser;
use mqtlib::config::mqtli_config::MqtliConfigBuilderError;
use mqtlib::config::mqtli_config::{
    LastWillConfigBuilderError, MqtliConfig, MqttBrokerConnectBuilderError,
};
use thiserror::Error;
use validator::{Validate, ValidationErrors};

#[derive(Error, Debug)]
pub enum ArgsError {
    #[error("Error while parsing broker args")]
    BrokerConfig(#[from] MqttBrokerConnectBuilderError),
    #[error("Error while parsing last will args")]
    LastWillConfig(#[from] LastWillConfigBuilderError),
    #[error("Error while parsing config args")]
    MqtliConfig(#[from] MqtliConfigBuilderError),
    #[error("Could not read config file \"{1}\"")]
    CouldNotReadConfigFile(#[source] io::Error, PathBuf),
    #[error("Could not parse config file \"{1}\"")]
    CouldNotParseConfigFile(#[source] serde_yaml::Error, PathBuf),
    #[error("Invalid configuration")]
    InvalidConfiguration(#[source] ValidationErrors),
}

pub fn load_config() -> Result<MqtliConfig, ArgsError> {
    let args = MqtliArgs::parse();

    let config_file_path = match &args.config_file {
        None => PathBuf::from("config.yaml"),
        Some(config_file) => config_file.to_path_buf(),
    };

    let mut config = MqtliConfig::default();

    match read_config_from_file(&config_file_path) {
        Ok(config_from_file) => config = config_from_file.merge(config)?,
        Err(err) => return Err(err),
    }

    config = args.merge(config)?;

    config
        .validate()
        .map(|_| config)
        .map_err(ArgsError::InvalidConfiguration)
}

fn read_config_from_file(buf: &PathBuf) -> Result<MqtliArgs, ArgsError> {
    let content = match read_to_string(buf) {
        Ok(content) => content,
        Err(e) => {
            return Err(ArgsError::CouldNotReadConfigFile(e, PathBuf::from(buf)));
        }
    };

    let config: MqtliArgs = match serde_yaml::from_str(content.as_str()) {
        Ok(config) => config,
        Err(e) => {
            return Err(ArgsError::CouldNotParseConfigFile(e, PathBuf::from(buf)));
        }
    };

    Ok(config)
}

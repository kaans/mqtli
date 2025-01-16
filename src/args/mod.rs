pub mod broker;
mod command;
pub mod content;
mod parsers;

use crate::args::command::Command;
use crate::args::content::MqtliArgs;
use clap::Parser;
use mqtlib::config::mqtli_config::MqtliConfigBuilderError;
use mqtlib::config::mqtli_config::{
    LastWillConfigBuilderError, MqtliConfig, MqttBrokerConnectBuilderError,
};
use mqtlib::config::publish::PublishBuilderError;
use mqtlib::config::subscription::SubscriptionBuilderError;
use mqtlib::config::topic::TopicBuilderError;
use std::fmt::Debug;
use std::fs::read_to_string;
use std::io;
use std::io::Read;
use std::path::PathBuf;
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
    #[error("Error while parsing topic args")]
    TopicConfig(#[from] TopicBuilderError),
    #[error("Error while parsing publish args")]
    PublishConfig(#[from] PublishBuilderError),
    #[error("Error while parsing subscription args")]
    SubscriptionConfig(#[from] SubscriptionBuilderError),
    #[error("Could not read config file \"{1}\"")]
    CouldNotReadConfigFile(#[source] io::Error, PathBuf),
    #[error("Could not parse config file \"{1}\"")]
    CouldNotParseConfigFile(#[source] serde_yaml::Error, PathBuf),
    #[error("Invalid configuration")]
    InvalidConfiguration(#[source] ValidationErrors),
    #[error("Error while reading data from stdin")]
    StdInError(#[from] io::Error),
}

pub fn load_config() -> Result<MqtliConfig, ArgsError> {
    let mut args = MqtliArgs::parse();
    let mut config = MqtliConfig::default();

    let config_file_path = match &args.config_file {
        None => PathBuf::from("config.yaml"),
        Some(config_file) => config_file.to_path_buf(),
    };

    match read_config_from_file(&config_file_path) {
        Ok(mut config_from_file) => {
            if let Some(command) = &args.command {
                match command {
                    Command::Publish(_) | Command::Subscribe(_) => {
                        config_from_file.topics.clear();
                    }
                    Command::Sparkplug(config) => {
                        if !config.include_topics_from_file {
                            config_from_file.topics.clear();
                        }
                    }
                }
            }
            config = config_from_file.merge(config)?;
        }
        Err(e) => match e {
            ArgsError::CouldNotReadConfigFile(_, _) => match args.command.as_ref() {
                Some(_) => {}
                _ => return Err(e),
            },
            _ => return Err(e),
        },
    };

    move_stdin_to_message(&mut args)?;

    config = args.merge(config)?;

    config
        .validate()
        .map(|_| config)
        .map_err(ArgsError::InvalidConfiguration)
}

fn move_stdin_to_message(args: &mut MqtliArgs) -> Result<(), io::Error> {
    if let Some(Command::Publish(ref mut publish_command)) = args.command {
        if publish_command.message.from_stdin {
            let stdin = io::stdin();
            let mut buf_from_stdin = Vec::new();
            stdin.lock().read_to_end(&mut buf_from_stdin)?;

            publish_command.message.message = Some(Box::new(buf_from_stdin));
        }
    }

    Ok(())
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

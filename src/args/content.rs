use crate::args::broker::MqttBrokerConnectArgs;
use crate::args::parsers::deserialize_level_filter;
use crate::args::ArgsError;

use crate::args::command::sql_storage::SqlStorage;
use crate::args::command::Command;
use clap::Parser;
use mqtlib::config::mqtli_config::{Mode, MqtliConfig, MqtliConfigBuilder};
use mqtlib::config::sql_storage::SqlStorage as SqlStorageConfig;
use mqtlib::config::topic::{Topic, TopicStorage};
use serde::Deserialize;
use std::path::PathBuf;
use tracing::Level;

#[derive(Debug, Deserialize, Parser)]
#[command(author, version, about, long_about = None)]
#[clap(disable_version_flag = true)]
#[clap(disable_help_flag = true)]
#[serde(deny_unknown_fields)]
pub struct MqtliArgs {
    #[clap(long, action = clap::ArgAction::HelpLong, help = "Print help")]
    help: Option<bool>,

    #[clap(long, action = clap::ArgAction::Version, help = "Print version")]
    version: Option<bool>,

    #[command(flatten)]
    pub broker: MqttBrokerConnectArgs,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_level_filter")]
    #[arg(
        short = 'l',
        long = "log-level",
        global = true,
        env = "LOG_LEVEL",
        help_heading = "Logging",
        help = "Log level (default: info) (possible values: trace, debug, info, warn, error, off)"
    )]
    pub log_level: Option<Level>,

    #[arg(
        short = 'c',
        long = "config-file",
        global = true,
        env = "CONFIG_FILE_PATH",
        help = "Path to the config file (default: config.yaml)"
    )]
    #[serde(skip_serializing)]
    pub config_file: Option<PathBuf>,

    #[clap(skip)]
    #[serde(default)]
    pub topics: Vec<Topic>,

    #[clap(subcommand)]
    #[serde(skip_serializing, skip_deserializing)]
    pub command: Option<Command>,

    #[clap(skip)]
    #[serde(default)]
    #[serde(rename = "database")]
    pub sql_storage: Option<SqlStorage>,
}

impl MqtliArgs {
    pub fn merge(self, other: MqtliConfig) -> Result<MqtliConfig, ArgsError> {
        let mut builder = MqtliConfigBuilder::default();

        let topics = self.assemble_topics(self.topics.clone())?;

        builder.broker(self.broker.merge(other.broker)?);

        builder.log_level(match self.log_level {
            None => other.log_level,
            Some(log_level) => log_level,
        });

        match self.command {
            None => {
                builder.mode(Mode::MultiTopic);
            }
            Some(command) => {
                match command {
                    Command::Publish(_) => builder.mode(Mode::Publish),
                    Command::Subscribe(_) => builder.mode(Mode::Subscribe),
                    Command::Sparkplug(_) => builder.mode(Mode::Sparkplug),
                };
            }
        };

        builder.topic_storage(TopicStorage {
            topics: other
                .topic_storage
                .topics
                .into_iter()
                .chain(topics)
                .collect(),
        });

        builder.sql_storage(match self.sql_storage {
            None => other.sql_storage,
            Some(sql) => {
                Some(SqlStorageConfig {
                    connection_string: sql.connection_string,
                })
            }
        });

        builder.build().map_err(ArgsError::from)
    }

    fn assemble_topics(
        &self,
        topics_from_config_file: Vec<Topic>,
    ) -> Result<Vec<Topic>, ArgsError> {
        let mut result = Vec::new();

        if let Some(command) = self.command.as_ref() {
            result.extend(command.get_topics()?);

            if let Command::Sparkplug(config) = command {
                if config.include_topics_from_file {
                    result.extend(topics_from_config_file);
                }
            }
        } else {
            result.extend(topics_from_config_file);
        }

        Ok(result)
    }
}

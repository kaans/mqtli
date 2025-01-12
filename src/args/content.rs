use crate::args::broker::MqttBrokerConnectArgs;
use crate::args::command::publish::Command;
use crate::args::parsers::deserialize_level_filter;
use crate::args::ArgsError;
use clap::Parser;
use mqtlib::config::filter::FilterTypes;
use mqtlib::config::mqtli_config::{Mode, MqtliConfig, MqtliConfigBuilder};
use mqtlib::config::publish::{PublishBuilder, PublishTriggerType, PublishTriggerTypePeriodic};
use mqtlib::config::topic::{Topic, TopicBuilder};
use mqtlib::config::{PayloadType, PublishInputType, PublishInputTypeContentPath};
use mqtlib::mqtt::QoS;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::Level;

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
    #[serde(skip_serializing)]
    pub command: Option<Command>,
}

impl MqtliArgs {
    pub fn merge(self, other: MqtliConfig) -> Result<MqtliConfig, ArgsError> {
        let mut builder = MqtliConfigBuilder::default();

        let command_topics = self.extract_topics_from_commands()?;

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
                };
            }
        };

        builder.topics(
            other
                .topics
                .into_iter()
                .chain(self.topics)
                .chain(command_topics)
                .collect(),
        );

        builder.build().map_err(ArgsError::from)
    }

    fn extract_topics_from_commands(&self) -> Result<Vec<Topic>, ArgsError> {
        let mut result = Vec::new();

        if let Some(command) = self.command.as_ref() {
            match command {
                Command::Publish(publish_command) => {
                    let trigger = PublishTriggerType::Periodic(PublishTriggerTypePeriodic::new(
                        publish_command.interval.unwrap_or(Duration::from_secs(1)),
                        publish_command.count.or(Some(1)),
                        Duration::from_millis(1000),
                    ));

                    let message_type = PublishInputTypeContentPath {
                        content: if publish_command.message.null_message {
                            None
                        } else if publish_command.message.message.is_some() {
                            Some(
                                publish_command
                                    .message
                                    .clone()
                                    .message
                                    .unwrap()
                                    .to_vec()
                                    .clone(),
                            )
                        } else {
                            None
                        },
                        path: if publish_command.message.file.is_some() {
                            publish_command.message.file.clone()
                        } else {
                            None
                        },
                    };

                    let message_input_type = match &publish_command.message_type {
                        None => PublishInputType::Text(message_type),
                        Some(payload_type) => match payload_type {
                            PublishInputType::Text(_) => PublishInputType::Text(message_type),
                            PublishInputType::Raw(_) => PublishInputType::Raw(message_type.into()),
                            PublishInputType::Hex(_) => PublishInputType::Hex(message_type),
                            PublishInputType::Json(_) => PublishInputType::Json(message_type),
                            PublishInputType::Yaml(_) => PublishInputType::Yaml(message_type),
                            PublishInputType::Base64(_) => PublishInputType::Base64(message_type),
                            PublishInputType::Null => {
                                PublishInputType::Text(PublishInputTypeContentPath::default())
                            }
                        },
                    };

                    let topic_type = publish_command
                        .topic_type
                        .clone()
                        .unwrap_or(PayloadType::Text);

                    let publish = PublishBuilder::default()
                        .qos(publish_command.qos.unwrap_or(QoS::AtLeastOnce))
                        .retain(publish_command.retain)
                        .enabled(true)
                        .trigger(vec![trigger])
                        .input(message_input_type)
                        .filters(FilterTypes::default())
                        .build()?;
                    let topic = TopicBuilder::default()
                        .topic(publish_command.topic.clone())
                        .publish(Some(publish))
                        .subscription(None)
                        .payload_type(topic_type)
                        .build()?;

                    result.push(topic);
                }
            }
        }

        Ok(result)
    }
}

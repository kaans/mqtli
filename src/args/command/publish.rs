use crate::args::parsers::parse_duration_milliseconds;
use crate::args::parsers::parse_qos;
use crate::args::parsers::parse_string_as_vec;
use clap::{Args, Subcommand};
use derive_getters::Getters;
use mqtlib::config::{PayloadType, PublishInputType};
use mqtlib::mqtt::QoS;
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Subcommand)]
pub enum Command {
    #[command(name = "pub")]
    Publish(CommandPublish),
}

#[derive(Args, Clone, Debug, Default, Deserialize, Getters)]
pub struct CommandPublish {
    #[arg(
        short = 't',
        long = "topic",
        env = "PUBLISH_TOPIC",
        help_heading = "Publish",
        help = "Topic to publish"
    )]
    pub topic: String,

    #[arg(short = 'q', long = "qos", env = "PUBLISH_QOS",
    value_parser = parse_qos,
    help_heading = "Publish",
    help = "Quality of Service (default: 0) (possible values: 0 = at most once; 1 = at least once; 2 = exactly once)"
    )]
    pub qos: Option<QoS>,

    #[arg(
        short = 'r',
        long = "retain",
        env = "PUBLISH_RETAIN",
        help_heading = "Publish",
        help = "If specified, the message is sent with the retain flag"
    )]
    pub retain: bool,

    #[arg(
        long = "message-type",
        env = "PUBLISH_MESSAGE_TYPE",
        help_heading = "Publish",
        help = "Payload type of the message"
    )]
    pub message_type: Option<PublishInputType>,

    #[arg(
        short = 'y',
        long = "topic-type",
        env = "PUBLISH_TOPIC_TYPE",
        help_heading = "Publish",
        help = "Payload type of the topic"
    )]
    pub topic_type: Option<PayloadType>,

    #[command(flatten)]
    pub message: CommandPublishMessage,

    #[arg(
        long = "interval",
        env = "PUBLISH_INTERVAL",
        value_parser = parse_duration_milliseconds,
        help_heading = "Publish",
        help = "Interval in milliseconds between two messages"
    )]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_milliseconds_from_option")]
    pub interval: Option<Duration>,

    #[arg(
        long = "repeat",
        env = "PUBLISH_REPEAT",
        help_heading = "Publish",
        help = "Repeat sending the message"
    )]
    pub count: Option<u32>,
}

#[derive(Args, Clone, Debug, Default, Deserialize, Getters)]
#[group(required = true, multiple = false)]
pub struct CommandPublishMessage {
    #[arg(
        short = 'm',
        long = "message",
        env = "PUBLISH_MESSAGE",
        value_parser = parse_string_as_vec,
        help_heading = "Publish",
        help = "Message to publish",
        group = "publish_message"
    )]
    #[allow(clippy::box_collection)]
    pub message: Option<Box<Vec<u8>>>,

    #[arg(
        short = 'n',
        long = "null-message",
        env = "PUBLISH_NULL_MESSAGE",
        help_heading = "Publish",
        help = "Sends a null (empty) message",
        group = "publish_message"
    )]
    pub null_message: bool,

    #[arg(
        short = 'f',
        long = "file",
        env = "PUBLISH_FILE",
        help_heading = "Publish",
        help = "Loads a message from a file",
        group = "publish_message"
    )]
    pub file: Option<PathBuf>,

    #[arg(
        long = "stdin-single",
        env = "PUBLISH_STDIN_ALL",
        help_heading = "Publish",
        help = "Read message from stdin and send content as a single message",
        group = "publish_message"
    )]
    pub from_stdin_single: bool,
}

fn deserialize_duration_milliseconds_from_option<'a, D>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'a>,
{
    let value: Option<u64> = Deserialize::deserialize(deserializer)?;
    Ok(value.map(Duration::from_millis))
}

#[cfg(test)]
mod tests {
    use crate::args::command::publish::Command;
    use crate::args::content::MqtliArgs;
    use clap::Parser;

    #[test]
    fn null() {
        let args = ["mqtli", "pub", "--topic", "TOPIC", "--null-message"];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.is_some());

        match result.command.unwrap() {
            Command::Publish(value) => {
                assert_eq!(value.topic, "TOPIC");
                assert!(value.message.null_message);
                assert!(value.message.message.is_none());
                assert!(value.message.file.is_none());
            }
        }
    }

    #[test]
    fn minimal() {
        let args = [
            "mqtli",
            "pub",
            "--topic",
            "TOPIC",
            "--message",
            "MESSAGE to send",
        ];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.is_some());

        match result.command.unwrap() {
            Command::Publish(value) => {
                assert_eq!(value.topic, "TOPIC");
                assert_eq!(
                    value.message.message.unwrap().to_vec(),
                    "MESSAGE to send".as_bytes()
                );
            }
        }
    }

    #[test]
    fn max() {
        let args = [
            "mqtli",
            "pub",
            "--topic",
            "TOPIC",
            "--message",
            "MESSAGE to send",
            "-q",
            "2",
            "-r",
        ];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.is_some());

        match result.command.unwrap() {
            Command::Publish(value) => {
                assert_eq!(value.topic, "TOPIC");
                assert_eq!(
                    value.message.message.unwrap().to_vec(),
                    "MESSAGE to send".as_bytes()
                );
            }
        }
    }

    #[test]
    fn invalid_qos() {
        let args = [
            "mqtli",
            "pub",
            "--topic",
            "TOPIC",
            "--message",
            "MESSAGE to send",
            "-q",
            "3",
        ];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_err());
    }

    #[test]
    fn pub_no_topic() {
        let args = ["mqtli", "pub", "-m \"MESSAGE to send\""];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_err());
    }

    #[test]
    fn pub_no_message() {
        let args = ["mqtli", "pub", "-t TOPIC"];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_err());
    }

    #[test]
    fn pub_empty() {
        let args = ["mqtli", "pub"];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_err());
    }
}

use crate::args::parsers::parse_duration_milliseconds;
use crate::args::parsers::parse_qos;
use crate::args::parsers::parse_string_as_vec;
use clap::Args;
use derive_getters::Getters;
use mqtlib::config::{PayloadType, PublishInputType};
use mqtlib::mqtt::QoS;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Args, Clone, Debug, Default, Getters)]
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
    pub interval: Option<Duration>,

    #[arg(
        long = "repeat",
        env = "PUBLISH_REPEAT",
        help_heading = "Publish",
        help = "Repeat sending the message"
    )]
    pub count: Option<u32>,
}

#[derive(Args, Clone, Debug, Default, Getters)]
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
        short = 's',
        long = "from-stdin",
        env = "PUBLISH_FROM_STDIN",
        help_heading = "Publish",
        help = "Read message from stdin and send content as a single message",
        group = "publish_message"
    )]
    pub from_stdin: bool,
}

#[cfg(test)]
mod tests {
    use crate::args::command::Command;
    use crate::args::content::MqtliArgs;
    use clap::Parser;

    #[test]
    fn null() {
        let args = ["mqtli", "pub", "--topic", "TOPIC", "--null-message"];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.is_some());

        if let Command::Publish(value) = result.command.unwrap() {
            assert_eq!(value.topic, "TOPIC");
            assert!(value.message.null_message);
            assert!(!value.message.from_stdin);
            assert!(value.message.message.is_none());
            assert!(value.message.file.is_none());
        }
    }

    #[test]
    fn file() {
        let args = ["mqtli", "pub", "--topic", "TOPIC", "--file", "filename"];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.is_some());

        if let Command::Publish(value) = result.command.unwrap() {
            assert_eq!(value.topic, "TOPIC");
            assert!(!value.message.null_message);
            assert!(!value.message.from_stdin);
            assert!(value.message.message.is_none());
            assert!(value.message.file.is_some());
        }
    }

    #[test]
    fn stdin() {
        let args = ["mqtli", "pub", "--topic", "TOPIC", "-s"];
        let result = MqtliArgs::try_parse_from(args);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.is_some());

        if let Command::Publish(value) = result.command.unwrap() {
            assert_eq!(value.topic, "TOPIC");
            assert!(!value.message.null_message);
            assert!(value.message.from_stdin);
            assert!(value.message.message.is_none());
            assert!(value.message.file.is_none());
        }
    }

    fn illegal_combination(a: &str, b: &str) -> Result<(), String> {
        let args = ["mqtli", "pub", "--topic", "TOPIC", a, b];
        let result = MqtliArgs::try_parse_from(args);

        if result.is_ok() {
            Err(format!("{} and {} must not be specified together", a, b))
        } else {
            Ok(())
        }
    }

    #[test]
    fn illegal_combinations() -> Result<(), String> {
        [
            ("-mTest", "-n"),
            ("-mTest", "-f"),
            ("-mTest", "-l"),
            ("-n", "-f"),
            ("-n", "-l"),
            ("-f", "-l"),
        ]
        .iter()
        .try_for_each(|(a, b)| illegal_combination(a, b))?;

        Ok(())
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

        if let Command::Publish(value) = result.command.unwrap() {
            assert_eq!(value.topic, "TOPIC");
            assert_eq!(
                value.message.message.unwrap().to_vec(),
                "MESSAGE to send".as_bytes()
            );
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

        if let Command::Publish(value) = result.command.unwrap() {
            assert_eq!(value.topic, "TOPIC");
            assert_eq!(
                value.message.message.unwrap().to_vec(),
                "MESSAGE to send".as_bytes()
            );
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

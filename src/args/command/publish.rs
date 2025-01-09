use clap::{Args, Subcommand};
use derive_getters::Getters;
use mqtlib::mqtt::QoS;
use serde::Deserialize;

#[derive(Debug, Deserialize, Subcommand)]
pub enum Command {
    #[command(name = "pub")]
    Publish(CommandPublish),
}

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct CommandPublish {
    #[arg(
        short = 't',
        long = "topic",
        env = "PUBLISH_TOPIC",
        help_heading = "Publish",
        help = "Topic to publish"
    )]
    pub topic: String,

    #[arg(short = 'q', long = "qos", env = "PUBLISH_QOS", value_parser = parse_qos,
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
        short = 'm',
        long = "message",
        env = "PUBLISH_MESSAGE",
        help_heading = "Publish",
        help = "Message to publish"
    )]
    pub message: String,
}

#[cfg(test)]
mod tests {
    use crate::args::command::publish::Command;
    use crate::args::content::MqtliArgs;
    use clap::Parser;

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
                assert_eq!(value.message, "MESSAGE to send");
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
                assert_eq!(value.message, "MESSAGE to send");
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

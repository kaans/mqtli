use crate::args::parsers::parse_qos;
use clap::{Args, Subcommand};
use mqtlib::config::PayloadType;
use mqtlib::mqtt::QoS;
use std::path::PathBuf;
use validator::Validate;

#[derive(Args, Clone, Debug, Default)]
pub struct CommandSubscribe {
    #[arg(
        short = 't',
        long = "topic",
        env = "SUBSCRIBE_TOPIC",
        help_heading = "Subscribe",
        help = "Topic to subscribe"
    )]
    pub topic: String,

    #[arg(short = 'q', long = "qos", env = "SUBSCRIBE_QOS",
    value_parser = parse_qos,
    help_heading = "Subscribe",
    help = "Quality of Service (default: 0) (possible values: 0 = at most once; 1 = at least once; 2 = exactly once)"
    )]
    pub qos: Option<QoS>,

    #[arg(
        short = 'y',
        long = "topic-type",
        env = "SUBSCRIBE_TOPIC_TYPE",
        help_heading = "Subscribe",
        help = "Payload type of the topic"
    )]
    pub topic_type: Option<PayloadType>,

    #[arg(
        long = "output-type",
        env = "SUBSCRIBE_OUTPUT_TYPE",
        help_heading = "Subscribe",
        help = "Payload type of the output"
    )]
    pub output_type: Option<PayloadType>,

    #[command(subcommand)]
    pub output_target: Option<OutputTarget>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum OutputTarget {
    #[command(name = "output-console")]
    Console(OutputTargetConsole),

    #[command(name = "output-file")]
    File(OutputTargetFile),

    #[command(name = "output-topic")]
    Topic(OutputTargetTopic),
}

impl Default for OutputTarget {
    fn default() -> Self {
        Self::Console(OutputTargetConsole::default())
    }
}

#[derive(Args, Clone, Debug, Default, PartialEq, Validate)]
pub struct OutputTargetConsole {}

#[derive(Args, Clone, Debug, Default, PartialEq, Validate)]
pub struct OutputTargetTopic {
    #[arg(
        id = "output-topic",
        long = "output-topic",
        env = "SUBSCRIBE_OUTPUT_TOPIC",
        help_heading = "Subscribe target topic",
        help = "Topic of the output"
    )]
    pub topic: String,

    #[arg(
        id = "output-qos",
        long = "output-qos",
        env = "SUBSCRIBE_OUTPUT_QOS",
        value_parser = parse_qos,
        help_heading = "Subscribe target topic",
        help = "QoS of the output"
    )]
    pub qos: Option<QoS>,

    #[arg(
        id = "output-retain",
        long = "output-retain",
        env = "SUBSCRIBE_OUTPUT_RETAIN",
        help_heading = "Subscribe target topic",
        help = "Retain of the output"
    )]
    pub retain: bool,
}

#[derive(Args, Clone, Debug, Default, PartialEq, Validate)]
pub struct OutputTargetFile {
    #[arg(
        id = "output-path",
        long = "output-path",
        env = "SUBSCRIBE_OUTPUT_PATH",
        help_heading = "Subscribe target file",
        help = "Path to the output file"
    )]
    pub path: PathBuf,

    #[arg(
        id = "output-overwrite",
        long = "output-overwrite",
        env = "SUBSCRIBE_OUTPUT_OVERWRITE",
        help_heading = "Subscribe target file",
        help = "Overwrite the output file with each message"
    )]
    pub overwrite: bool,

    #[arg(
        id = "output-prepend",
        long = "output-prepend",
        env = "SUBSCRIBE_OUTPUT_PREPEND",
        help_heading = "Subscribe target file",
        help = "Prepend the output with this"
    )]
    pub prepend: Option<String>,

    #[arg(
        id = "output-append",
        long = "output-append",
        env = "SUBSCRIBE_OUTPUT_APPEND",
        help_heading = "Subscribe target file",
        help = "Append the output with this"
    )]
    pub append: Option<String>,
}

use crate::args::parsers::parse_qos;
use clap::Args;
use derive_getters::Getters;
use mqtlib::config::PayloadType;
use mqtlib::mqtt::QoS;
use serde::Deserialize;

#[derive(Args, Clone, Debug, Default, Deserialize, Getters)]
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
}

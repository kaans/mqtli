use crate::args::parsers::parse_qos;
use clap::Args;
use mqtlib::mqtt::QoS;
use mqtlib::sparkplug::GroupId;

#[derive(Args, Clone, Debug, Default)]
pub struct CommandSparkplug {
    /*
    TODO ideas for config values:

    - subscribe only to certain groups
    - exclude only certain groups from subscription
    - subscribe only to certain edge nodes from a specific group
    - exclude certain edge nodes from a specific groups
    - decide if only sparkplug commands should be printed, or also other topics
       -> this implies that all subscriptions must be disabled/enabled includidng their output functions
    */
    #[arg(
        short = 'q',
        long = "qos",
        env = "SPARKPLUG_QOS",
        value_parser = parse_qos,
        help_heading = "Sparkplug",
        help = "Quality of Service (default: 0) (possible values: 0 = at most once; 1 = at least once; 2 = exactly once)"
    )]
    pub qos: Option<QoS>,

    #[arg(
        long = "include-topics-from-file",
        env = "SPARKPLUG_QOS",
        help_heading = "Sparkplug",
        help = "Include topics defined in the config file"
    )]
    pub include_topics_from_file: bool,

    #[arg(
        long = "include-group",
        alias = "ig",
        env = "SPARKPLUG_INCLUDE_GROUPS",
        value_delimiter = ',',
        help_heading = "Sparkplug",
        help = "Include only the given topics; if not specified, all groups are subscribed to"
    )]
    pub include_groups: Vec<GroupId>,
}

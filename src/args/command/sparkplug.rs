use crate::args::parsers::parse_qos;
use clap::Args;
use mqtlib::mqtt::QoS;

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
    #[arg(short = 'q', long = "qos", env = "SPARKPLUG_QOS",
    value_parser = parse_qos,
    help_heading = "Sparkplug",
    help = "Quality of Service (default: 0) (possible values: 0 = at most once; 1 = at least once; 2 = exactly once)"
    )]
    pub qos: Option<QoS>,
}

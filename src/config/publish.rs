use crate::config::{args, PublishInputType};
use crate::mqtt::QoS;
use derive_getters::Getters;
use std::time::Duration;
use validator::Validate;

#[derive(Debug, Getters, Validate)]
pub struct Publish {
    enabled: bool,
    qos: QoS,
    retain: bool,
    trigger: Vec<PublishTriggerType>,
    #[validate(nested)]
    input: PublishInputType,
}

impl Default for Publish {
    fn default() -> Self {
        Publish {
            enabled: true,
            qos: Default::default(),
            retain: false,
            trigger: vec![],
            input: Default::default(),
        }
    }
}

impl From<&args::Publish> for Publish {
    fn from(value: &args::Publish) -> Self {
        let trigger: Vec<PublishTriggerType> = match value.trigger() {
            None => {
                vec![PublishTriggerType::default()]
            }
            Some(trigger) => trigger.iter().map(PublishTriggerType::from).collect(),
        };

        Publish {
            enabled: *value.enabled(),
            qos: *value.qos(),
            retain: *value.retain(),
            trigger,
            input: (*value.input()).clone(),
        }
    }
}

#[derive(Debug, Getters, Validate)]
pub struct PublishTriggerTypePeriodic {
    interval: Duration,
    count: Option<u32>,
    initial_delay: Duration,
}

impl Default for PublishTriggerTypePeriodic {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
            count: None,
            initial_delay: Duration::from_secs(0),
        }
    }
}

impl From<&args::PublishTriggerTypePeriodic> for PublishTriggerTypePeriodic {
    fn from(value: &args::PublishTriggerTypePeriodic) -> Self {
        let default = Self::default();

        Self {
            interval: match value.interval() {
                None => default.interval,
                Some(value) => *value,
            },
            count: match value.count() {
                None => default.count,
                Some(value) => Some(*value),
            },
            initial_delay: match value.initial_delay() {
                None => default.initial_delay,
                Some(value) => *value,
            },
        }
    }
}

#[derive(Debug)]
pub enum PublishTriggerType {
    Periodic(PublishTriggerTypePeriodic),
}

impl From<&args::PublishTriggerType> for PublishTriggerType {
    fn from(value: &args::PublishTriggerType) -> Self {
        match value {
            args::PublishTriggerType::Periodic(value) => {
                PublishTriggerType::Periodic(PublishTriggerTypePeriodic::from(value))
            }
        }
    }
}

impl Default for PublishTriggerType {
    fn default() -> Self {
        Self::Periodic(PublishTriggerTypePeriodic::default())
    }
}

use crate::config::args::deserialize_qos;
use crate::config::filter::{FilterError, FilterTypes};
use crate::config::PublishInputType;
use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use derive_getters::Getters;
use serde::{Deserialize, Deserializer};
use std::time::Duration;
use validator::Validate;

#[derive(Clone, Debug, Deserialize, Getters, Validate)]
pub struct Publish {
    #[serde(default)]
    enabled: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
    #[serde(default)]
    retain: bool,
    #[serde(default)]
    trigger: Vec<PublishTriggerType>,
    #[validate(nested)]
    input: PublishInputType,
    #[serde(default)]
    filters: FilterTypes,
}

impl Publish {
    pub fn apply_filters(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        self.filters.apply(data)
    }
}

impl Default for Publish {
    fn default() -> Self {
        Publish {
            enabled: true,
            qos: Default::default(),
            retain: false,
            trigger: vec![],
            input: Default::default(),
            filters: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Getters, Validate)]
pub struct PublishTriggerTypePeriodic {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_milliseconds")]
    interval: Duration,
    count: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_milliseconds")]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PublishTriggerType {
    #[serde(rename = "periodic")]
    Periodic(PublishTriggerTypePeriodic),
}

impl Default for PublishTriggerType {
    fn default() -> Self {
        Self::Periodic(PublishTriggerTypePeriodic::default())
    }
}

fn deserialize_duration_milliseconds<'a, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'a>,
{
    let value: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_millis(value))
}

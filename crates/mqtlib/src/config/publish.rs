use crate::config::deserialize_qos;
use crate::config::filter::{FilterError, FilterTypes};
use crate::config::PublishInputType;
use crate::mqtt::QoS;
use crate::payload::{PayloadFormat, PayloadFormatError};
use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use validator::Validate;

#[derive(Builder, Clone, Debug, Deserialize, Getters, Validate)]
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

impl Display for Publish {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Enabled: {}", self.enabled)?;
        writeln!(f, "QoS: {}", self.qos)?;
        writeln!(f, "Retain: {}", self.retain)?;
        writeln!(f, "Input: {}", self.input)?;

        writeln!(f, "Triggers:")?;
        self.trigger()
            .iter()
            .enumerate()
            .map(|(i, trigger)| writeln!(f, "{i}. {}", trigger))
            .collect::<Result<Vec<_>, fmt::Error>>()?;

        writeln!(f, "Filters:")?;
        self.filters
            .0
            .iter()
            .enumerate()
            .map(|(i, filter)| writeln!(f, "{i}. {}", filter))
            .collect::<Result<Vec<_>, fmt::Error>>()?;

        Ok(())
    }
}

impl TryFrom<Publish> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_from(value: Publish) -> Result<Self, Self::Error> {
        PayloadFormat::try_from(&value.input)
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

#[derive(Builder, Clone, Debug, Deserialize, Getters, Validate, new)]
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
            initial_delay: Duration::from_millis(1000),
        }
    }
}

#[derive(Clone, Debug, Deserialize, strum_macros::Display)]
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

pub fn deserialize_duration_milliseconds<'a, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'a>,
{
    let value: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_millis(value))
}

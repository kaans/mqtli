use crate::config::deserialize_qos;
use crate::config::filter::{FilterError, FilterTypes};
use crate::config::PayloadType;
use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use derive_builder::Builder;
use derive_getters::Getters;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use validator::Validate;

#[derive(Builder, Clone, Debug, Deserialize, Getters, PartialEq, Validate)]
pub struct Subscription {
    enabled: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
    outputs: Vec<Output>,
    #[serde(default)]
    filters: FilterTypes,
}

impl Subscription {
    pub fn apply_filters(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        self.filters.apply(data)
    }
}

impl Display for Subscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Enabled: {}", self.enabled)?;
        writeln!(f, "QoS: {}", self.qos)?;

        for (i, output) in self.outputs.iter().enumerate() {
            writeln!(f, "Output: {i}\n{}", output)?;
        }

        Ok(())
    }
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            enabled: true,
            qos: Default::default(),
            outputs: vec![],
            filters: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq, Validate)]
pub struct Output {
    format: PayloadType,
    #[serde(default)]
    target: OutputTarget,
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "format: {}", self.format)?;
        writeln!(f, "target: {}", self.target)?;

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, strum_macros::Display, PartialEq)]
#[serde(tag = "type")]
pub enum OutputTarget {
    #[serde(rename = "console")]
    Console(OutputTargetConsole),
    #[serde(rename = "file")]
    File(OutputTargetFile),
    #[serde(rename = "topic")]
    Topic(OutputTargetTopic),
}

impl Default for OutputTarget {
    fn default() -> Self {
        OutputTarget::Console(OutputTargetConsole::default())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq, Validate)]
pub struct OutputTargetConsole {}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq, Validate)]
pub struct OutputTargetTopic {
    topic: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos")]
    qos: QoS,
    #[serde(default)]
    retain: bool,
}

#[derive(Clone, Debug, Deserialize, Getters, PartialEq, Validate)]
pub struct OutputTargetFile {
    path: PathBuf,
    #[serde(default)]
    overwrite: bool,
    prepend: Option<String>,
    append: Option<String>,
}

impl Default for OutputTargetFile {
    fn default() -> Self {
        OutputTargetFile {
            path: Default::default(),
            overwrite: false,
            prepend: None,
            append: Some("\n".to_string()),
        }
    }
}

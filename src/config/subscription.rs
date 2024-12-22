use crate::config::filter::{FilterError, FilterImpl, FilterType};
use crate::config::{args, PayloadType};
use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use derive_getters::Getters;
use log::debug;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use validator::Validate;

#[derive(Debug, Getters, Validate)]
pub struct Subscription {
    enabled: bool,
    qos: QoS,
    outputs: Vec<Output>,
    filters: Vec<FilterType>,
}

impl Subscription {
    pub fn apply_filters(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        debug!("Applying filters {:?}", self.filters);

        let result: Result<Vec<PayloadFormat>, FilterError> =
            self.filters
                .iter()
                .try_fold(vec![data], |payloads, filter| {
                    let result: Result<Vec<PayloadFormat>, FilterError> = payloads
                        .iter()
                        .map(|payload| FilterImpl::apply(filter, payload.clone()))
                        .try_fold(vec![], |mut unrolled, result| {
                            unrolled.extend(result?);
                            Ok(unrolled)
                        });

                    result
                });

        result
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
        Subscription {
            enabled: true,
            qos: Default::default(),
            outputs: vec![],
            filters: vec![],
        }
    }
}

impl From<&args::Subscription> for Subscription {
    fn from(value: &args::Subscription) -> Self {
        let outputs: Vec<Output> = match value.outputs() {
            None => {
                vec![Output::default()]
            }
            Some(outputs) => outputs.iter().map(Output::from).collect(),
        };

        let filters: Vec<FilterType> = match value.filters() {
            None => {
                vec![FilterType::default()]
            }
            Some(filters) => filters.to_vec(),
        };

        Subscription {
            enabled: *value.enabled(),
            qos: *value.qos(),
            outputs,
            filters,
        }
    }
}

#[derive(Debug, Default, Getters, Validate)]
pub struct Output {
    format: PayloadType,
    target: OutputTarget,
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "format: {}", self.format)?;
        writeln!(f, "target: {}", self.target)?;

        Ok(())
    }
}

impl From<&args::Output> for Output {
    fn from(value: &args::Output) -> Self {
        Output {
            format: match value.format() {
                None => PayloadType::default(),
                Some(value) => value.clone(),
            },
            target: match value.target() {
                None => OutputTarget::Console(OutputTargetConsole::default()),
                Some(value) => match value {
                    args::OutputTarget::Console(options) => {
                        OutputTarget::Console(OutputTargetConsole::from(options))
                    }
                    args::OutputTarget::File(options) => {
                        OutputTarget::File(OutputTargetFile::from(options))
                    }
                },
            },
        }
    }
}

#[derive(Debug, strum_macros::Display)]
pub enum OutputTarget {
    Console(OutputTargetConsole),
    File(OutputTargetFile),
}

impl Default for OutputTarget {
    fn default() -> Self {
        OutputTarget::Console(OutputTargetConsole::default())
    }
}

#[derive(Debug, Default, Getters, Validate)]
pub struct OutputTargetConsole {}

impl From<&args::OutputTargetConsole> for OutputTargetConsole {
    fn from(_: &args::OutputTargetConsole) -> Self {
        Self {}
    }
}

#[derive(Debug, Getters, Validate)]
pub struct OutputTargetFile {
    path: PathBuf,
    overwrite: bool,
    prepend: Option<String>,
    append: Option<String>,
}

impl From<&args::OutputTargetFile> for OutputTargetFile {
    fn from(value: &args::OutputTargetFile) -> Self {
        Self {
            path: PathBuf::from(value.path()),
            overwrite: *value.overwrite(),
            prepend: value.prepend().clone(),
            append: value.append().clone(),
        }
    }
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

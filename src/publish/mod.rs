use thiserror::Error;

use crate::payload::PayloadFormatError;

pub mod trigger_periodic;

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error("Scheduler is already running, cannot add more jobs")]
    SchedulerAlreadyRunning,
    #[error("Could not convert payload")]
    CouldNotConvertPayload(#[source] PayloadFormatError),
}

impl From<PayloadFormatError> for TriggerError {
    fn from(value: PayloadFormatError) -> Self {
        Self::CouldNotConvertPayload(value)
    }
}
use thiserror::Error;
use tokio_cron_scheduler::JobSchedulerError;

use crate::payload::PayloadFormatError;

pub mod trigger_periodic;

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error("Could not convert payload")]
    CouldNotConvertPayload(#[source] PayloadFormatError),
    #[error("Job scheduling error")]
    JobSchedulerError(#[from] JobSchedulerError),
}

impl From<PayloadFormatError> for TriggerError {
    fn from(value: PayloadFormatError) -> Self {
        Self::CouldNotConvertPayload(value)
    }
}

use thiserror::Error;
use crate::config::ConfigError;

pub mod trigger_periodic;

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error("Scheduler is already running, cannot add more jobs")]
    SchedulerAlreadyRunning,
    #[error("Could not convert payload")]
    CouldNotConvertPayload(#[source] ConfigError),
}

use thiserror::Error;

use crate::input::InputError;

pub mod trigger_periodic;

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error("Scheduler is already running, cannot add more jobs")]
    SchedulerAlreadyRunning,
    #[error("Could not convert payload")]
    CouldNotConvertPayload(#[source] InputError),
}

use thiserror::Error;

pub mod trigger_periodic;

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error("Scheduler is already running, cannot add more jobs")]
    SchedulerAlreadyRunning
}

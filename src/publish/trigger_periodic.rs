use std::time::Duration;

use clokwerk::{AsyncScheduler, Interval, Job};
use log::info;
use tokio::task::JoinHandle;

use crate::publish::TriggerError;

pub struct TriggerPeriodic {
    scheduler: Option<Box<AsyncScheduler>>,
    task_handle: Option<JoinHandle<()>>,
}

impl TriggerPeriodic {
    pub fn new() -> Self {
        Self {
            scheduler: Option::from(Box::new(AsyncScheduler::new())),
            task_handle: None,
        }
    }

    pub fn add_schedule(&mut self,
                        interval: &Duration,
                        count: &Option<u32>,
                        initial_delay: &Duration) -> Result<(), TriggerError> {
        if let Some(scheduler) = self.scheduler.as_mut() {
            let mut job = scheduler.every(Interval::Seconds(interval.as_secs() as u32))
                .plus(Interval::Seconds(initial_delay.as_secs() as u32));

            if let Some(count) = count {
                job = job.count(*count as usize);
            } else {
                job = job.forever();
            }

            let intv = interval.clone();
            let cnt = count.clone();

            job.run(move || async move {
                println!("I'm repeated async every {:?} seconds for {:?} times", intv, cnt);
            });

            Ok(())
        } else {
            return Err(TriggerError::SchedulerAlreadyRunning);
        }
    }

    pub async fn start(mut self) {
        if self.scheduler.is_some() {
            let mut scheduler = self.scheduler.unwrap();

            self.task_handle = Some(tokio::spawn(async move {
                loop {
                    scheduler.run_pending().await;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }));
        } else {
            info!("Not starting scheduler, not jobs scheduled");
        }
    }
}


use std::hash::{Hash, Hasher};
use chrono::{DateTime, Utc};
use crate::sparkplug::{DeviceId, Status};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SparkplugDevice {
    // id fields
    pub device_id: Option<DeviceId>,
    pub metric_levels: Vec<String>,

    // status attributes
    pub status: Status,
    pub last_status_update: DateTime<Utc>
}

impl Hash for SparkplugDevice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.device_id.hash(state);
        self.metric_levels.hash(state);
    }
}

impl Eq for SparkplugDevice {}

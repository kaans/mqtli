use crate::sparkplug::MessageStorage;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SparkplugHostApplication {
    pub host_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct SparkplugHostApplicationStorage(
    pub(crate) HashMap<SparkplugHostApplication, MessageStorage>,
);

impl SparkplugHostApplicationStorage {
    pub fn count_received_messages(&self) -> usize {
        self.0.values().map(|e| e.messages.len()).sum()
    }
}

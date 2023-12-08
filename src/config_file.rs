use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Getters)]
pub struct ConfigFile {
    subscribe_topics: Vec<String>
}
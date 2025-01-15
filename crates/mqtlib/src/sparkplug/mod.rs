use crate::payload::sparkplug::PayloadFormatSparkplug;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use strum_macros::{Display, EnumString};
use thiserror::Error;

type GroupId = String;
type EdgeNodeId = String;
type DeviceId = String;

pub const SPARKPLUG_TOPIC_VERSION: &str = "spBv1.0";

#[derive(Debug, Error)]
pub enum SparkplugError {
    #[error(
        "Topic did not contain at least 4 parts (namespace, group id, message type, edge node id)"
    )]
    NotEnoughPartsInTopic,
    #[error("Version of topic must be {version}", version = SPARKPLUG_TOPIC_VERSION)]
    InvalidTopicVersion,
    #[error("Message type not valid")]
    InvalidTopicMessageType,
    #[error("Group id contains invalid characters")]
    GroupIdNotValid,
    #[error("Edge node id contains invalid characters")]
    EdgeNodeIdNotValid,
    #[error("Device id contains invalid characters")]
    DeviceIdNotValid,
}

#[derive(Clone, Debug, Default)]
pub struct SparkplugNetwork {
    pub host_applications: SparkplugHostApplicationStorage,
    pub edge_nodes: SparkplugEdgeNodeStorage,
}

#[derive(Clone, Debug, Default)]
pub struct SparkplugHostApplicationStorage(HashMap<SparkplugHostApplication, MessageStorage>);

impl SparkplugHostApplicationStorage {
    pub fn count_received_messages(&self) -> usize {
        self.0.values().map(|e| e.messages.len()).sum()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SparkplugEdgeNodeStorage(HashMap<SparkplugEdgeNode, MessageStorage>);

impl SparkplugEdgeNodeStorage {
    pub fn count_received_messages(&self) -> usize {
        self.0.values().map(|e| e.messages.len()).sum()
    }

    pub fn list_group_ids(&self) -> HashSet<GroupId> {
        self.0.keys().map(|e| e.group_id.clone()).collect()
    }

    pub fn find_by_group_id(&self, group_id: GroupId) -> HashSet<&SparkplugEdgeNode> {
        self.0
            .keys()
            .filter(|e| e.group_id == group_id)
            .collect()
    }

    pub fn find_by_edge_node_id(&self, group_id: GroupId, edge_node_id: EdgeNodeId) -> HashSet<&SparkplugEdgeNode> {
        self.0
            .keys()
            .filter(|e| e.group_id == group_id)
            .filter(|e| e.edge_node_id == edge_node_id)
            .collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct MessageStorage {
    pub messages: Vec<PayloadFormatSparkplug>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SparkplugHostApplication {
    pub host_id: String,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct SparkplugEdgeNode {
    pub group_id: GroupId,
    pub edge_node_id: EdgeNodeId,
    pub device_id: Option<DeviceId>,
    pub metric_levels: Vec<String>,
}

impl SparkplugNetwork {
    pub fn count_received_messages(&self) -> usize {
        self.edge_nodes.count_received_messages() + self.host_applications.count_received_messages()
    }

    pub fn try_parse_message(
        &mut self,
        topic: String,
        message: PayloadFormatSparkplug,
    ) -> Result<(), SparkplugError> {
        let topic = SparkplugTopic::try_from(topic)?;

        match topic {
            SparkplugTopic::EdgeNode(data) => {
                let edge_node = SparkplugEdgeNode {
                    group_id: data.group_id,
                    edge_node_id: data.edge_node_id,
                    device_id: data.device_id,
                    metric_levels: data.metric_levels,
                };

                let storage = self.edge_nodes.0.entry(edge_node).or_default();
                storage.messages.push(message);
            }
            SparkplugTopic::HostApplication(data) => {
                let host = SparkplugHostApplication {
                    host_id: data.host_id,
                };

                let storage = self.host_applications.0.entry(host).or_default();
                storage.messages.push(message);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Display, EnumString)]
pub enum SparkplugMessageType {
    NBIRTH,
    NDATA,
    NDEATH,
    DBIRTH,
    DDATA,
    DDEATH,
    NCMD,
    DCMD,
    STATE,
}

#[derive(Clone)]
pub enum SparkplugTopic {
    EdgeNode(SparkplugTopicEdgeNode),
    HostApplication(SparkplugTopicHostApplication),
}

#[derive(Clone)]
pub struct SparkplugTopicEdgeNode {
    pub version: String,
    pub group_id: String,
    pub edge_node_id: String,
    pub message_type: SparkplugMessageType,
    pub device_id: Option<String>,
    pub metric_levels: Vec<String>,
}

#[derive(Clone)]
pub struct SparkplugTopicHostApplication {
    pub version: String,
    pub host_id: String,
    pub message_type: SparkplugMessageType,
}

impl TryFrom<String> for SparkplugTopic {
    type Error = SparkplugError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        fn is_part_valid(part: &str) -> bool {
            !part.contains(['+', '/', '#'])
        }

        let split: Vec<&str> = value.split('/').collect();

        if split.len() < 3 {
            return Err(SparkplugError::NotEnoughPartsInTopic);
        }

        match split[1] {
            "STATE" => {
                let Ok(message_type) = SparkplugMessageType::try_from(split[1]) else {
                    return Err(SparkplugError::InvalidTopicMessageType);
                };

                Ok(Self::HostApplication(SparkplugTopicHostApplication {
                    version: split[0].to_string(),
                    host_id: split[2].to_string(),
                    message_type,
                }))
            }
            &_ => {
                if split.len() < 4 {
                    Err(SparkplugError::NotEnoughPartsInTopic)
                } else {
                    if split[0] != SPARKPLUG_TOPIC_VERSION {
                        return Err(SparkplugError::InvalidTopicVersion);
                    }

                    if !is_part_valid(split[1]) {
                        return Err(SparkplugError::GroupIdNotValid);
                    }
                    if !is_part_valid(split[2]) {
                        return Err(SparkplugError::EdgeNodeIdNotValid);
                    }

                    let Ok(message_type) = SparkplugMessageType::try_from(split[2]) else {
                        return Err(SparkplugError::InvalidTopicMessageType);
                    };

                    let device_id = if split.len() > 4 {
                        if !is_part_valid(split[4]) {
                            return Err(SparkplugError::DeviceIdNotValid);
                        }
                        Some(split[4].to_string())
                    } else {
                        None
                    };
                    let metric_levels = split[5..].iter().map(|s| s.to_string()).collect();

                    Ok(Self::EdgeNode(SparkplugTopicEdgeNode {
                        version: split[0].to_string(),
                        group_id: split[1].to_string(),
                        edge_node_id: split[3].to_string(),
                        message_type,
                        device_id,
                        metric_levels,
                    }))
                }
            }
        }
    }
}

impl Display for SparkplugTopic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SparkplugTopic::EdgeNode(data) => {
                write!(
                    f,
                    "{}/{}/{}/{}",
                    data.version, data.group_id, data.message_type, data.edge_node_id
                )?;

                if let Some(device_id) = &data.device_id {
                    write!(f, "/{}", device_id)?;
                }

                for part in &data.metric_levels {
                    write!(f, "/{}", part)?;
                }

                Ok(())
            }
            SparkplugTopic::HostApplication(data) => {
                write!(f, "{}/{}/{}", data.version, data.message_type, data.host_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equals() {
        let a = get_edge_node();
        let b = get_edge_node();
        assert_eq!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.device_id = Some("a".to_string());
        b.device_id = Some("b".to_string());
        assert_ne!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.device_id = Some("a".to_string());
        b.device_id = Some("a".to_string());
        a.metric_levels = vec!["a".to_string()];
        b.metric_levels = vec!["a".to_string()];
        assert_eq!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.device_id = None;
        b.device_id = None;
        a.metric_levels = vec!["a".to_string()];
        b.metric_levels = vec!["a".to_string()];
        assert_eq!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.device_id = Some("a".to_string());
        b.device_id = Some("a".to_string());
        a.metric_levels = vec!["a".to_string()];
        b.metric_levels = vec!["b".to_string()];
        assert_ne!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.device_id = None;
        b.device_id = None;
        a.metric_levels = vec!["a".to_string()];
        b.metric_levels = vec!["b".to_string()];
        assert_ne!(a, b);
    }

    fn get_edge_node() -> SparkplugEdgeNode {
        SparkplugEdgeNode {
            group_id: "group".to_string(),
            edge_node_id: "edge".to_string(),
            device_id: None,
            metric_levels: vec![],
        }
    }
}

use crate::payload::sparkplug::PayloadFormatSparkplug;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use strum_macros::{Display, EnumString};
use thiserror::Error;

type GroupId = String;
type EdgeNodeId = String;
type DeviceId = String;

#[derive(Debug, Error)]
pub enum SparkplugError {
    #[error(
        "Topic did not contain at least 4 parts (namespace, group id, message type, edge node id)"
    )]
    NotEnoughPartsInTopic,
    #[error("Version of topic must be spBv1.0")]
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
    pub host_applications: HashSet<SparkplugHostApplication>,
    pub edge_nodes: HashSet<SparkplugEdgeNode>,
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
    pub fn try_parse_message(
        &mut self,
        topic: String,
        message: PayloadFormatSparkplug,
    ) -> Result<(), SparkplugError> {
        self.add_to_network_by_topic(topic)?;

        Ok(())
    }

    pub fn groups(&self) -> HashSet<GroupId> {
        self.edge_nodes.iter().map(|e| e.group_id.clone()).collect()
    }

    fn add_to_network_by_topic(&mut self, topic: String) -> Result<(), SparkplugError> {
        let topic = SparkplugTopic::try_from(topic)?;

        match topic {
            SparkplugTopic::EdgeNode(data) => {
                let edge_node = SparkplugEdgeNode {
                    group_id: data.group_id,
                    edge_node_id: data.edge_node_id,
                    device_id: data.device_id,
                    metric_levels: data.metric_levels,
                };

                let edge_node = if let Some(existing) = self.edge_nodes.take(&edge_node) {
                    existing
                } else {
                    edge_node
                };

                self.edge_nodes.insert(edge_node);
            }
            SparkplugTopic::HostApplication(data) => {
                let host = SparkplugHostApplication {
                    host_id: data.host_id,
                };

                self.host_applications.insert(host);
            }
        }
        Ok(())
    }
}

#[derive(Display, EnumString)]
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

pub enum SparkplugTopic {
    EdgeNode(SparkplugTopicEdgeNode),
    HostApplication(SparkplugTopicHostApplication),
}

pub struct SparkplugTopicEdgeNode {
    pub version: String,
    pub group_id: String,
    pub edge_node_id: String,
    pub message_type: SparkplugMessageType,
    pub device_id: Option<String>,
    pub metric_levels: Vec<String>,
}

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
                    if split[0] != "spBv1.0" {
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

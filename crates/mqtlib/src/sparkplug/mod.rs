pub mod device;
pub mod edge_node;
pub mod host_application;
pub mod network;
pub mod topic;

use crate::payload::sparkplug::PayloadFormatSparkplug;
use strum_macros::{Display, EnumString};
use thiserror::Error;

pub type GroupId = String;
pub type EdgeNodeId = String;
pub type DeviceId = String;

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
pub struct MessageStorage {
    pub messages: Vec<PayloadFormatSparkplug>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Status {
    ONLINE,

    #[default]
    OFFLINE,
}

#[derive(Clone, Display, EnumString, PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::sparkplug::device::SparkplugDevice;
    use crate::sparkplug::edge_node::SparkplugEdgeNode;

    #[test]
    fn equals() {
        let a = get_edge_node();
        let b = get_edge_node();
        assert_eq!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.devices.push(SparkplugDevice {
            device_id: Some("a".to_string()),
            ..Default::default()
        });
        b.devices.push(SparkplugDevice {
            device_id: Some("b".to_string()),
            ..Default::default()
        });
        assert_ne!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.devices.push(SparkplugDevice {
            device_id: Some("a".to_string()),
            ..Default::default()
        });
        b.devices.push(SparkplugDevice {
            device_id: Some("a".to_string()),
            ..Default::default()
        });
        assert_eq!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.devices.push(SparkplugDevice {
            device_id: Some("a".to_string()),
            ..Default::default()
        });
        b.devices.push(SparkplugDevice {
            device_id: Some("a".to_string()),
            ..Default::default()
        });
        assert_eq!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.devices.push(SparkplugDevice {
            device_id: None,
            ..Default::default()
        });
        b.devices.push(SparkplugDevice {
            device_id: None,
            ..Default::default()
        });
        assert_eq!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.devices.push(SparkplugDevice {
            device_id: Some("a".to_string()),
            metric_levels: vec!["a".to_string()],
            ..Default::default()
        });
        b.devices.push(SparkplugDevice {
            device_id: Some("a".to_string()),
            metric_levels: vec!["b".to_string()],
            ..Default::default()
        });
        assert_ne!(a, b);

        let mut a = get_edge_node();
        let mut b = get_edge_node();
        a.devices.push(SparkplugDevice {
            device_id: None,
            metric_levels: vec!["a".to_string()],
            ..Default::default()
        });
        b.devices.push(SparkplugDevice {
            device_id: None,
            metric_levels: vec!["b".to_string()],
            ..Default::default()
        });
        assert_ne!(a, b);
    }

    fn get_edge_node() -> SparkplugEdgeNode {
        SparkplugEdgeNode {
            group_id: "group".to_string(),
            edge_node_id: "edge".to_string(),
            ..Default::default()
        }
    }
}

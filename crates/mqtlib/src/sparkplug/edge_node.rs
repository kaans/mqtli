use crate::payload::sparkplug::protos::sparkplug_b::payload::Template;
use crate::sparkplug::device::SparkplugDevice;
use crate::sparkplug::{EdgeNodeId, GroupId, MessageStorage, Status};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SparkplugEdgeNode {
    // id fields
    pub group_id: GroupId,
    pub edge_node_id: EdgeNodeId,
    pub devices: Vec<SparkplugDevice>,

    // status attributes
    pub templates: HashMap<String, Template>,
    pub status: Status,
    pub last_status_update: DateTime<Utc>,
}

impl SparkplugEdgeNode {
    pub fn new(group_id: GroupId, edge_node_id: EdgeNodeId) -> Self {
        SparkplugEdgeNode {
            group_id,
            edge_node_id,
            ..Default::default()
        }
    }
}

impl Hash for SparkplugEdgeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.group_id.hash(state);
        self.edge_node_id.hash(state);
    }
}

impl Eq for SparkplugEdgeNode {}

#[derive(Clone, Debug, Default)]
pub struct SparkplugEdgeNodeStorage(HashMap<SparkplugEdgeNode, MessageStorage>);

impl SparkplugEdgeNodeStorage {
    pub(crate) fn get_message_storage(
        &mut self,
        group_id: String,
        edge_node_id: String,
    ) -> &mut MessageStorage {
        let edge_node = SparkplugEdgeNode {
            group_id,
            edge_node_id,
            ..Default::default()
        };

        self.0.entry(edge_node).or_default()
    }
}

impl SparkplugEdgeNodeStorage {
    pub fn count_received_messages(&self) -> usize {
        self.0.values().map(|e| e.messages.len()).sum()
    }

    pub fn list_group_ids(&self) -> HashSet<GroupId> {
        self.0.keys().map(|e| e.group_id.clone()).collect()
    }

    #[allow(clippy::mutable_key_type)]
    pub fn find_by_group_id(&self, group_id: GroupId) -> HashSet<&SparkplugEdgeNode> {
        self.0.keys().filter(|e| e.group_id == group_id).collect()
    }

    #[allow(clippy::mutable_key_type)]
    pub fn find_by_edge_node_id(
        &self,
        group_id: &GroupId,
        edge_node_id: &EdgeNodeId,
    ) -> Option<&SparkplugEdgeNode> {
        let mut edge_nodes: HashSet<&SparkplugEdgeNode> = self
            .0
            .keys()
            .filter(|e| &e.group_id == group_id)
            .filter(|e| &e.edge_node_id == edge_node_id)
            .collect();

        edge_nodes.iter().next().cloned()
    }

    #[allow(clippy::mutable_key_type)]
    pub fn find_by_edge_node_id_or_create(
        &mut self,
        group_id: &GroupId,
        edge_node_id: &EdgeNodeId,
    ) -> &SparkplugEdgeNode {
        if self.find_by_edge_node_id(group_id, edge_node_id).is_none() {
            let edge_node_new = SparkplugEdgeNode::new(group_id.clone(), edge_node_id.clone());
            self.0.insert(edge_node_new, MessageStorage::default());
        }

        self.find_by_edge_node_id(group_id, edge_node_id).unwrap()
    }

    #[allow(clippy::mutable_key_type)]
    pub fn set_status(
        &mut self,
        group_id: &GroupId,
        edge_node_id: &EdgeNodeId,
        status: Status,
    ) {
        if let Some(mut edge_node) = self.find_by_edge_node_id(group_id, edge_node_id) {
            //edge_node.status = status;
        }
    }
}

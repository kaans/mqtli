use crate::payload::sparkplug::protos::sparkplug_b::payload::metric::Value;
use crate::payload::sparkplug::protos::sparkplug_b::payload::Template;
use crate::payload::sparkplug::PayloadFormatSparkplug;
use crate::sparkplug::edge_node::SparkplugEdgeNodeStorage;
use crate::sparkplug::host_application::{
    SparkplugHostApplication, SparkplugHostApplicationStorage,
};
use crate::sparkplug::topic::SparkplugTopic;
use std::collections::HashMap;
use tracing::{debug, trace, warn};

#[derive(Clone, Debug, Default)]
pub struct SparkplugNetwork {
    pub host_applications: SparkplugHostApplicationStorage,
    pub edge_nodes: SparkplugEdgeNodeStorage,
}

impl SparkplugNetwork {
    pub fn count_received_messages(&self) -> usize {
        self.edge_nodes.count_received_messages() + self.host_applications.count_received_messages()
    }

    pub fn parse_message(&mut self, topic: SparkplugTopic, message: PayloadFormatSparkplug) {
        match topic {
            SparkplugTopic::EdgeNode(data) => {
                let storage = self
                    .edge_nodes
                    .get_message_storage(data.group_id, data.edge_node_id);
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
    }

    fn _extract_templates(&self, message: &PayloadFormatSparkplug) -> HashMap<String, Template> {
        let mut result = HashMap::new();

        for metric in &message.content.metrics {
            if let Some(Value::TemplateValue(template)) = &metric.value {
                if template.is_definition() {
                    match &metric.name {
                        None => {
                            warn!("Ignoring template definition because it doesn't have a name");
                            trace!("Offending template definition: {}", template);
                        }
                        Some(name) => {
                            debug!("Found template definition {name}");
                            result.insert(name.clone(), template.clone());
                        }
                    }
                }
            }
        }

        result
    }
}

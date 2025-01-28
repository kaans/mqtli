use std::fmt::{Display, Formatter};
use crate::sparkplug::{SparkplugError, SparkplugMessageType, SPARKPLUG_TOPIC_VERSION};

#[derive(Clone)]
pub enum SparkplugTopic {
    EdgeNode(SparkplugTopicEdgeNode),
    HostApplication(SparkplugTopicHostApplication),
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
                    let mut metric_levels = vec![];

                    if split.len() > 5 {
                        split[5..]
                            .iter()
                            .for_each(|s| metric_levels.push(s.to_string()));
                    }

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

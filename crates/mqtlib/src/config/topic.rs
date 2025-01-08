use crate::config::publish::Publish;
use crate::config::subscription::Subscription;
use crate::config::PayloadType;
use derive_builder::Builder;
use derive_getters::Getters;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use validator::Validate;

#[derive(Builder, Clone, Debug, Default, Deserialize, Getters, Validate)]
pub struct Topic {
    #[validate(length(min = 1, message = "Topic must be given"))]
    topic: String,
    #[validate(nested)]
    subscription: Option<Subscription>,
    #[serde(default)]
    #[serde(rename = "payload")]
    payload_type: PayloadType,
    #[validate(nested)]
    pub publish: Option<Publish>,
}

impl Topic {
    /// Checks if the given topic is contained in this topic considering all wildcards.
    pub(crate) fn contains(&self, rhs: &str) -> bool {
        if self.topic == rhs {
            return true;
        }

        let parts_self: Vec<&str> = self.topic.split("/").collect();
        let parts_rhs: Vec<&str> = rhs.split("/").collect();

        let result = parts_self
            .iter()
            .enumerate()
            .zip(parts_rhs.iter().enumerate())
            .map(|((l_i, &l), (r_i, &r))| {
                let is_last_on_either_side = (l_i == parts_self.len() - 1
                    && parts_self.len() < parts_rhs.len())
                    || (r_i == parts_rhs.len() - 1 && parts_rhs.len() < parts_self.len());

                ((l == r || l == "+") && !is_last_on_either_side) || l == "#"
            })
            .all(|part| part);

        result
    }
}

impl Display for Topic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "topic: {}", self.topic)?;
        writeln!(f, "payload type: {}", self.payload_type)?;
        writeln!(
            f,
            "Subscription:\n{}",
            self.subscription
                .as_ref()
                .map_or("None".to_string(), |value| value.to_string())
        )?;
        writeln!(
            f,
            "Publish:\n{}",
            self.publish
                .as_ref()
                .map_or("None".to_string(), |value| value.to_string())
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_contains() {
        let topic = Topic {
            topic: "the/topic".to_string(),
            subscription: Default::default(),
            payload_type: Default::default(),
            publish: None,
        };

        assert_eq!(true, topic.contains("the/topic"));
        assert_eq!(false, topic.contains("the/topik"));
        assert_eq!(false, topic.contains("toolong/the/topic"));
        assert_eq!(false, topic.contains("/the/topic"));
        assert_eq!(false, topic.contains("the/topic/toolong"));
        assert_eq!(false, topic.contains("the/topic/"));
    }

    #[test]
    fn topic_contains_single_wildcard() {
        let topic = get_topic("the/topic/+");

        assert_eq!(true, topic.contains("the/topic/something"));
        assert_eq!(true, topic.contains("the/topic/"));
        assert_eq!(false, topic.contains("/the/topic"));
        assert_eq!(false, topic.contains("the/topic"));
        assert_eq!(false, topic.contains("the/topik/something"));
        assert_eq!(false, topic.contains("/the/topic/something"));
    }

    #[test]
    fn topic_contains_two_wildcards() {
        let topic = get_topic("the/topic/+/is/+/longer");

        assert_eq!(true, topic.contains("the/topic/something/is/alot/longer"));
        assert_eq!(
            false,
            topic.contains("the/topic/something/is/alot/longeeee")
        );
        assert_eq!(false, topic.contains("zhe/topic/something/is/alot/longer"));
        assert_eq!(true, topic.contains("the/topic//is//longer"));
        assert_eq!(false, topic.contains("/the/topic/something/is/alot/longer"));
        assert_eq!(false, topic.contains("the/topic/is/longer"));
        assert_eq!(false, topic.contains("the/topik/something"));
        assert_eq!(false, topic.contains("/the/topic/something"));
    }

    #[test]
    fn topic_contains_all_wildcard() {
        let topic = get_topic("the/topic/#");

        assert_eq!(true, topic.contains("the/topic/something"));
        assert_eq!(true, topic.contains("the/topic/something/is/alot/longer"));
        assert_eq!(true, topic.contains("the/topic/"));
        assert_eq!(true, topic.contains("the/topic//////"));
        assert_eq!(false, topic.contains("/the/topic"));
        assert_eq!(false, topic.contains("the/topic"));
        assert_eq!(false, topic.contains("the/topik/something"));
        assert_eq!(false, topic.contains("/the/topic/something"));
    }

    fn get_topic(topic: &str) -> Topic {
        Topic {
            topic: topic.to_string(),
            subscription: Default::default(),
            payload_type: Default::default(),
            publish: None,
        }
    }
}

use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::mqtt::QoS;
use derive_getters::Getters;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use validator::{Validate, ValidationError, ValidationErrors};

pub mod filter;
pub mod mqtli_config;
pub mod publish;
pub mod subscription;
pub mod topic;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PayloadType {
    #[serde(rename = "text")]
    #[default]
    Text,
    #[serde(rename = "protobuf")]
    Protobuf(PayloadProtobuf),
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "yaml")]
    Yaml,
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "base64")]
    Base64,
    #[serde(rename = "raw")]
    Raw,
    #[serde(rename = "sparkplug")]
    Sparkplug,
}

impl Display for PayloadType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadType::Protobuf(value) => {
                write!(f, "Protobuf [Options: {}]", value)
            }
            PayloadType::Text => {
                write!(f, "Text")
            }
            PayloadType::Json => {
                write!(f, "Json")
            }
            PayloadType::Yaml => {
                write!(f, "Yaml")
            }
            PayloadType::Hex => {
                write!(f, "Hex")
            }
            PayloadType::Base64 => {
                write!(f, "Base64")
            }
            PayloadType::Raw => {
                write!(f, "Raw")
            }
            PayloadType::Sparkplug => write!(f, "Sparkplug"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadProtobuf {
    definition: PathBuf,
    message: String,
}

impl Display for PayloadProtobuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "definition: {:?}", self.definition)?;
        write!(f, "message: {:?}", self.message)
    }
}

#[derive(Clone, Debug, Deserialize, strum_macros::Display)]
#[serde(tag = "type")]
pub enum PublishInputType {
    #[serde(rename = "text")]
    Text(PublishInputTypeContentPath),
    #[serde(rename = "raw")]
    Raw(PublishInputTypePath),
    #[serde(rename = "hex")]
    Hex(PublishInputTypeContentPath),
    #[serde(rename = "json")]
    Json(PublishInputTypeContentPath),
    #[serde(rename = "yaml")]
    Yaml(PublishInputTypeContentPath),
    #[serde(rename = "base64")]
    Base64(PublishInputTypeContentPath),
}

impl Default for PublishInputType {
    fn default() -> Self {
        Self::Text(PublishInputTypeContentPath::default())
    }
}

impl Validate for PublishInputType {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            PublishInputType::Text(value) => {
                ValidationErrors::merge(Ok(()), "Text", value.validate())
            }
            PublishInputType::Raw(value) => {
                ValidationErrors::merge(Ok(()), "Raw", value.validate())
            }
            PublishInputType::Hex(value) => {
                ValidationErrors::merge(Ok(()), "Hex", value.validate())
            }
            PublishInputType::Json(value) => {
                ValidationErrors::merge(Ok(()), "Json", value.validate())
            }
            PublishInputType::Yaml(value) => {
                ValidationErrors::merge(Ok(()), "Yaml", value.validate())
            }
            PublishInputType::Base64(value) => {
                ValidationErrors::merge(Ok(()), "Base64", value.validate())
            }
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters)]
pub struct PublishInputTypeContentPath {
    content: Option<String>,
    path: Option<PathBuf>,
}

impl Validate for PublishInputTypeContentPath {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut err = ValidationError::new("invalid_publish_input");

        if (self.path.is_none() && self.content.is_none())
            || (self.path.is_some() && self.content.is_some())
        {
            err.message = Some(Cow::from(
                "Exactly one of path or content must be given for publish input",
            ));
            let mut errors = ValidationErrors::new();
            errors.add("content", err);
            return Err(errors);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, Validate)]
pub struct PublishInputTypePath {
    path: PathBuf,
}

pub fn deserialize_qos<'a, D>(deserializer: D) -> Result<QoS, D::Error>
where
    D: Deserializer<'a>,
{
    let value: Result<u8, _> = Deserialize::deserialize(deserializer);

    if let Ok(int_value) = value {
        return Ok(match int_value {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        });
    }

    Err(Error::invalid_value(
        Unexpected::Other("unknown"),
        &"unsigned integer between 0 and 2",
    ))
}

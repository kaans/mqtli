use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use derive_getters::Getters;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use strum_macros::EnumString;
use validator::{Validate, ValidationError, ValidationErrors};

pub mod filter;
pub mod mqtli_config;
pub mod publish;
pub mod sql_storage;
pub mod subscription;
pub mod topic;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, EnumString)]
#[serde(tag = "type")]
pub enum PayloadType {
    #[serde(rename = "text")]
    #[strum(serialize = "text")]
    #[default]
    Text,
    #[serde(rename = "protobuf")]
    #[strum(serialize = "protobuf")]
    Protobuf(PayloadProtobuf),
    #[serde(rename = "json")]
    #[strum(serialize = "json")]
    Json,
    #[serde(rename = "yaml")]
    #[strum(serialize = "yaml")]
    Yaml,
    #[serde(rename = "hex")]
    #[strum(serialize = "hex")]
    Hex,
    #[serde(rename = "base64")]
    #[strum(serialize = "base64")]
    Base64,
    #[serde(rename = "raw")]
    #[strum(serialize = "raw")]
    Raw,
    #[serde(rename = "sparkplug")]
    #[strum(serialize = "sparkplug")]
    Sparkplug,
    #[serde(rename = "sparkplug_json")]
    #[strum(serialize = "sparkplug_json")]
    SparkplugJson,
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
            PayloadType::SparkplugJson => write!(f, "Sparkplug Json"),
        }
    }
}

impl From<PayloadFormat> for PayloadType {
    fn from(value: PayloadFormat) -> Self {
        match value {
            PayloadFormat::Text(_) => PayloadType::Text,
            PayloadFormat::Raw(_) => PayloadType::Raw,
            PayloadFormat::Protobuf(_) => PayloadType::Protobuf(Default::default()),
            PayloadFormat::Hex(_) => PayloadType::Hex,
            PayloadFormat::Base64(_) => PayloadType::Base64,
            PayloadFormat::Json(_) => PayloadType::Json,
            PayloadFormat::Yaml(_) => PayloadType::Yaml,
            PayloadFormat::Sparkplug(_) => PayloadType::Sparkplug,
            PayloadFormat::SparkplugJson(_) => PayloadType::SparkplugJson,
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

#[derive(Clone, Debug, Deserialize, strum_macros::Display, EnumString)]
#[serde(tag = "type")]
pub enum PublishInputType {
    #[serde(rename = "text")]
    #[strum(serialize = "text")]
    Text(PublishInputTypeContentPath),
    #[serde(rename = "raw")]
    #[strum(serialize = "raw")]
    Raw(PublishInputTypePath),
    #[serde(rename = "hex")]
    #[strum(serialize = "hex")]
    Hex(PublishInputTypeContentPath),
    #[serde(rename = "json")]
    #[strum(serialize = "json")]
    Json(PublishInputTypeContentPath),
    #[serde(rename = "yaml")]
    #[strum(serialize = "yaml")]
    Yaml(PublishInputTypeContentPath),
    #[serde(rename = "base64")]
    #[strum(serialize = "base64")]
    Base64(PublishInputTypeContentPath),
    #[serde(rename = "null")]
    #[strum(serialize = "null")]
    Null,
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
            PublishInputType::Null => ValidationErrors::merge(Ok(()), "Null", Ok(())),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters)]
pub struct PublishInputTypeContentPath {
    #[serde(deserialize_with = "parse_string_as_vec")]
    pub content: Option<Vec<u8>>,
    pub path: Option<PathBuf>,
}

fn parse_string_as_vec<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    Ok(Some(value.into_bytes()))
}

impl Validate for PublishInputTypeContentPath {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut err = ValidationError::new("invalid_publish_input");

        if self.path.is_some() && self.content.is_some() {
            err.message = Some(Cow::from(
                "Exactly one of path or content must be given for publish input, or none for a null message",
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

impl From<PublishInputTypeContentPath> for PublishInputTypePath {
    fn from(value: PublishInputTypeContentPath) -> Self {
        Self {
            path: value.path.unwrap(),
        }
    }
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

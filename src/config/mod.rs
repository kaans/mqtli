use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

use derive_getters::Getters;
use serde::Deserialize;
use thiserror::Error;
use validator::{Validate, ValidationError, ValidationErrors};

mod args;
pub mod mqtli_config;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file \"{1}\"")]
    CouldNotReadConfigFile(#[source] io::Error, PathBuf),
    #[error("Could not parse config file \"{1}\"")]
    CouldNotParseConfigFile(#[source] serde_yaml::Error, PathBuf),
    #[error("Invalid configuration")]
    InvalidConfiguration(#[source] ValidationErrors),
}

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

#[derive(Clone, Debug, Deserialize)]
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

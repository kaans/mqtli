use std::borrow::Cow;
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PayloadType {
    #[serde(rename = "text")]
    Text(PayloadText),
    #[serde(rename = "protobuf")]
    Protobuf(PayloadProtobuf),
    #[serde(rename = "json")]
    Json(PayloadJson),
    #[serde(rename = "yaml")]
    Yaml(PayloadYaml),
    #[serde(rename = "hex")]
    Hex(PayloadHex),
    #[serde(rename = "base64")]
    Base64(PayloadBase64),
    #[serde(rename = "raw")]
    Raw(PayloadRaw),
}

impl Default for PayloadType {
    fn default() -> Self {
        PayloadType::Text(PayloadText::default())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadText {
    #[serde(default)]
    #[serde(rename = "raw_as")]
    raw_as_type: PayloadOptionRawFormat,
}

impl PayloadText {
    pub fn new(raw_as_type: PayloadOptionRawFormat) -> Self {
        Self { raw_as_type }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadProtobuf {
    definition: PathBuf,
    message: String,
}

/// The format to which bytes get decoded to.
/// Default is hex.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub enum PayloadOptionRawFormat {
    #[default]
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "base64")]
    Base64,
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadJson {
    #[serde(default)]
    #[serde(rename = "raw_as")]
    raw_as_type: PayloadOptionRawFormat,
}

impl PayloadJson {
    pub fn new(raw_as_type: PayloadOptionRawFormat) -> Self {
        Self { raw_as_type }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadYaml {
    #[serde(default)]
    #[serde(rename = "raw_as")]
    raw_as_type: PayloadOptionRawFormat,
}

impl PayloadYaml {
    pub fn new(raw_as_type: PayloadOptionRawFormat) -> Self {
        Self { raw_as_type }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadHex {}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadBase64 {}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct PayloadRaw {}

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

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use protofish::context::ParseError;
use thiserror::Error;

use crate::config::mqtli_config::{PayloadType, PublishInputType, PublishInputTypeContentPath};
use crate::config::OutputFormat;
use crate::payload::base64::PayloadFormatBase64;
use crate::payload::hex::PayloadFormatHex;
use crate::payload::json::PayloadFormatJson;
use crate::payload::protobuf::{PayloadFormatProtobuf, PayloadFormatProtobufInput};
use crate::payload::raw::PayloadFormatRaw;
use crate::payload::text::PayloadFormatText;
use crate::payload::yaml::PayloadFormatYaml;

pub mod text;
pub mod raw;
pub mod protobuf;
pub mod hex;
pub mod base64;
pub mod json;
pub mod yaml;

#[derive(Debug, Error)]
pub enum PayloadFormatError {
    #[error("Could not convert payload to UTF 8 string")]
    CouldNotConvertToUtf8(#[source] FromUtf8Error),
    #[error("Conversion from format {0} to format {1} not possible")]
    ConversionNotPossible(String, String),
    #[error("Cannot read content from path {1}")]
    CannotReadInputFromPath(#[source] io::Error, PathBuf),
    #[error("Either content or path to content must be given")]
    EitherContentOrPathMustBeGiven,
    #[error("Could not open definition file {0}")]
    CouldNotOpenDefinitionFile(String),
    #[error("Could not parse proto file {0}")]
    CouldNotParseProtoFile(#[source] ParseError),
    #[error("Message {0} not found in proto file, cannot decode payload")]
    MessageNotFoundInProtoFile(String),
    #[error("Invalid protobuf")]
    InvalidProtobuf,
    #[error("Field with number {0} not found in proto file")]
    FieldNumberNotFoundInProtoFile(u64),
    #[error("Could not convert payload to yaml")]
    CouldNotConvertToYaml(#[source] serde_yaml::Error),
    #[error("Could not convert payload to json")]
    CouldNotConvertToJson(#[source] serde_json::Error),
}

impl From<FromUtf8Error> for PayloadFormatError {
    fn from(value: FromUtf8Error) -> Self {
        Self::CouldNotConvertToUtf8(value)
    }
}

impl From<serde_json::Error> for PayloadFormatError {
    fn from(value: serde_json::Error) -> Self {
        Self::CouldNotConvertToJson(value)
    }
}

impl From<serde_yaml::Error> for PayloadFormatError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::CouldNotConvertToYaml(value)
    }
}

#[derive(Debug)]
pub enum PayloadFormat {
    Text(PayloadFormatText),
    Raw(PayloadFormatRaw),
    Protobuf(PayloadFormatProtobuf),
    Hex(PayloadFormatHex),
    Base64(PayloadFormatBase64),
    Json(PayloadFormatJson),
    Yaml(PayloadFormatYaml),
}

impl TryInto<Vec<u8>> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self {
            PayloadFormat::Text(value) => Ok(value.into()),
            PayloadFormat::Raw(value) => Ok(value.into()),
            PayloadFormat::Protobuf(value) => Ok(value.into()),
            PayloadFormat::Hex(value) => Ok(value.into()),
            PayloadFormat::Base64(value) => Ok(value.into()),
            PayloadFormat::Json(value) => Ok(value.into()),
            PayloadFormat::Yaml(value) => value.try_into(),
        }
    }
}

pub struct PayloadFormatOutput {
    output_format: OutputFormat,
    content: Vec<u8>,
}

impl PayloadFormatOutput {
    pub fn new(output_format: OutputFormat, content: Vec<u8>) -> Self {
        Self {
            output_format,
            content,
        }
    }
}

impl TryFrom<PayloadFormatOutput> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatOutput) -> Result<Self, Self::Error> {
        Ok(match value.output_format {
            OutputFormat::Text => PayloadFormat::Text(PayloadFormatText::try_from(value.content)?),
            OutputFormat::Json => PayloadFormat::Json(PayloadFormatJson::try_from(value.content)?),
            OutputFormat::Yaml => PayloadFormat::Yaml(PayloadFormatYaml::try_from(value.content)?),
            OutputFormat::Hex => PayloadFormat::Hex(PayloadFormatHex::from(value.content)),
            OutputFormat::Base64 => PayloadFormat::Base64(PayloadFormatBase64::from(value.content)),
            OutputFormat::Raw => PayloadFormat::Raw(PayloadFormatRaw::try_from(value.content)?),
        })
    }
}

impl PayloadFormat {
    pub fn new(input: &PublishInputType, output: &PayloadType) -> Result<PayloadFormat, PayloadFormatError> {
        let content = match input {
            PublishInputType::Text(input) => {
                read_input_type_content_path(input)?
            }
            PublishInputType::Raw(input) => {
                read_from_path(input.path())?
            }
        };

        match output {
            PayloadType::Text(_options) => {
                Ok(PayloadFormat::Text(PayloadFormatText::try_from(content)?))
            }
            PayloadType::Protobuf(options) => {
                let param = PayloadFormatProtobufInput::new(
                    content,
                    options.definition().clone(),
                    options.message().clone(),
                );

                Ok(PayloadFormat::Protobuf(PayloadFormatProtobuf::try_from(
                    param
                )?))
            }
        }
    }
}

fn read_input_type_content_path(input: &PublishInputTypeContentPath) -> Result<Vec<u8>, PayloadFormatError> {
    if let Some(content) = input.content() {
        Ok(Vec::from(content.as_str()))
    } else {
        if let Some(path) = input.path() {
            read_from_path(path)
        } else {
            return Err(PayloadFormatError::EitherContentOrPathMustBeGiven);
        }
    }
}

fn read_from_path(path: &PathBuf) -> Result<Vec<u8>, PayloadFormatError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { return Err(PayloadFormatError::CannotReadInputFromPath(e, PathBuf::from(path))); }
    };

    let mut buf = Vec::new();
    if let Err(e) = file.read_to_end(&mut buf) {
        return Err(PayloadFormatError::CannotReadInputFromPath(e, PathBuf::from(path)));
    };
    Ok(buf)
}

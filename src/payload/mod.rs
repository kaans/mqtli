use std::str::Utf8Error;

use log::error;
use protofish::context::ParseError;
use thiserror::Error;

pub mod protobuf;
pub mod text;

#[derive(Debug, Error)]
pub enum PayloadError {
    #[error("Could not convert payload to UTF 8 string")]
    CouldNotConvertToUtf8(#[source] Utf8Error),
    #[error("Could not open definition file {0}")]
    CouldNotOpenDefinitionFile(String),
    #[error("Could not parse proto file {0}")]
    CouldNotParseProtoFile(#[source] ParseError),
    #[error("Message {0} not found in proto file, cannot decode payload")]
    MessageNotFoundInProtoFile(String),
    #[error("Field with number {0} not found in proto file")]
    FieldNumberNotFoundInProtoFile(u64),
    #[error("Could not convert payload to yaml")]
    CouldNotConvertToYaml(#[source] serde_yaml::Error),
}
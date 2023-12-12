use std::str::Utf8Error;

use log::error;
use protofish::context::ParseError;
use thiserror::Error;

pub mod protobuf;
pub mod text;

#[derive(Debug)]
pub enum OutputFormat {
    PLAIN,
    JSON,
    YAML,
    CSV,
}

#[derive(Debug, Error)]
pub enum PayloadError {
    #[error("Output format {0:?} not supported")]
    OutputFormatNotSupported(OutputFormat),
    #[error("Could not convert payload to UTF 8 string")]
    CouldNotConvertToUtf8(Utf8Error),
    #[error("Could not open definition file {0}")]
    CouldNotOpenDefinitionFile(String),
    #[error("Could not parse proto file {0}")]
    CouldNotParseProtoFile(ParseError),
    #[error("Message {0} not found in proto file, cannot decode payload")]
    MessageNotFoundInProtoFile(String),
    #[error("Field with number {0} not found in proto file")]
    FieldNumberNotFoundInProtoFile(u64),
}
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use ::base64::DecodeError;
use ::hex::FromHexError;
use log::error;
use protobuf_json_mapping::PrintError;
use strum_macros::IntoStaticStr;
use thiserror::Error;

use crate::config::filter::FilterError;
use crate::config::{PayloadType, PublishInputType, PublishInputTypeContentPath};
use crate::payload::base64::PayloadFormatBase64;
use crate::payload::hex::PayloadFormatHex;
use crate::payload::json::PayloadFormatJson;
use crate::payload::protobuf::PayloadFormatProtobuf;
use crate::payload::raw::PayloadFormatRaw;
use crate::payload::text::PayloadFormatText;
use crate::payload::yaml::PayloadFormatYaml;

pub mod base64;
pub mod hex;
pub mod json;
pub mod protobuf;
pub mod raw;
pub mod text;
pub mod yaml;

#[derive(Debug, Error)]
pub enum PayloadFormatError {
    #[error("Could not convert payload to UTF 8 string")]
    CouldNotConvertToUtf8(#[source] FromUtf8Error),
    #[error("Conversion from format {0} to format {1} not possible")]
    ConversionNotPossible(String, String),
    #[error("Display of format {0} is not possible")]
    DisplayNotPossible(String),
    #[error("Cannot read content from path {1}")]
    CannotReadInputFromPath(#[source] io::Error, PathBuf),
    #[error("Either content or path to content must be given")]
    EitherContentOrPathMustBeGiven,
    #[error("Could not open definition file {0}")]
    CouldNotOpenDefinitionFile(String),
    #[error("Could not open protobuf definition file")]
    CouldNotOpenProtobufDefinitionFile,
    #[error("Message {0} not found in proto file, cannot decode payload")]
    MessageNotFoundInProtoFile(String),
    #[error("Invalid protobuf")]
    InvalidProtobuf,
    #[error("Protobuf message {0} not found")]
    ProtobufMessageNotFound(String),
    #[error("Field with number {0} not found in proto file")]
    FieldNumberNotFoundInProtoFile(u64),
    #[error("Could not convert payload to yaml")]
    CouldNotConvertToYaml(#[source] serde_yaml::Error),
    #[error("Could not convert payload from yaml")]
    CouldNotConvertFromYaml(String),
    #[error("Could not convert payload to json")]
    CouldNotConvertToJson(#[source] serde_json::Error),
    #[error("Could not convert payload from json")]
    CouldNotConvertFromJson(String),
    #[error("Could not convert payload from protobuf to format {0}")]
    CouldNotConvertFromProtobuf(&'static str),
    #[error("Could not convert payload to hex")]
    CouldNotConvertToHex(#[source] FromHexError),
    #[error("Could not convert payload to base64")]
    CouldNotConvertToBase64(#[source] DecodeError),
    #[error("The value is not valid hex formatted: {0}")]
    ValueIsNotValidHex(String),
    #[error("The value is not valid base64 formatted: {0}")]
    ValueIsNotValidBase64(String),
    #[error("Error while converting protobuf to JSON: {0}")]
    ProtobufJsonConversionError(#[from] PrintError),
    #[error("Error while parsing protobuf: {0}")]
    ProtobufParseError(#[from] ::protobuf::Error),
    #[error("Error while parsing protobuf from JSON: {0}")]
    ProtobufJsonMappingError(#[from] protobuf_json_mapping::ParseError),
    #[error("Error while applying filters")]
    FilterError(#[from] FilterError),
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

impl From<FromHexError> for PayloadFormatError {
    fn from(value: FromHexError) -> Self {
        Self::CouldNotConvertToHex(value)
    }
}

impl From<DecodeError> for PayloadFormatError {
    fn from(value: DecodeError) -> Self {
        Self::CouldNotConvertToBase64(value)
    }
}

#[derive(IntoStaticStr, Clone, Debug)]
pub enum PayloadFormat {
    Text(PayloadFormatText),
    Raw(PayloadFormatRaw),
    Protobuf(PayloadFormatProtobuf),
    Hex(PayloadFormatHex),
    Base64(PayloadFormatBase64),
    Json(PayloadFormatJson),
    Yaml(PayloadFormatYaml),
}

impl Display for PayloadFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(f, "{}", name)
    }
}
impl TryInto<Vec<u8>> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self {
            PayloadFormat::Text(value) => Ok(value.into()),
            PayloadFormat::Raw(value) => Ok(value.into()),
            PayloadFormat::Protobuf(value) => Ok(value.try_into()?),
            PayloadFormat::Hex(value) => Ok(value.into()),
            PayloadFormat::Base64(value) => Ok(value.into()),
            PayloadFormat::Json(value) => Ok(value.into()),
            PayloadFormat::Yaml(value) => value.try_into(),
        }
    }
}

impl TryInto<String> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            PayloadFormat::Text(value) => Ok(value.into()),
            PayloadFormat::Raw(value) => {
                Ok(String::from_utf8_lossy(Vec::<u8>::from(value).as_slice()).to_string())
            }
            PayloadFormat::Protobuf(_value) => Err(PayloadFormatError::DisplayNotPossible(
                String::from("protobuf"),
            )),
            PayloadFormat::Hex(value) => Ok(value.into()),
            PayloadFormat::Base64(value) => Ok(value.into()),
            PayloadFormat::Json(value) => Ok(value.into()),
            PayloadFormat::Yaml(value) => value.try_into(),
        }
    }
}

impl TryFrom<(PayloadFormat, &PayloadType)> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_from((value, payload_type): (PayloadFormat, &PayloadType)) -> Result<Self, Self::Error> {
        Ok(match payload_type {
            PayloadType::Text => PayloadFormat::Text(PayloadFormatText::try_from(value)?),
            PayloadType::Json => PayloadFormat::Json(PayloadFormatJson::try_from(value)?),
            PayloadType::Yaml => PayloadFormat::Yaml(PayloadFormatYaml::try_from(value)?),
            PayloadType::Hex => PayloadFormat::Hex(PayloadFormatHex::try_from(value)?),
            PayloadType::Base64 => PayloadFormat::Base64(PayloadFormatBase64::try_from(value)?),
            PayloadType::Raw => PayloadFormat::Raw(PayloadFormatRaw::try_from(value)?),
            PayloadType::Protobuf(options) => {
                PayloadFormat::Protobuf(PayloadFormatProtobuf::try_from((value, options))?)
            }
        })
    }
}

/// Converts the data given in the Vec<u8> to the corresponding payload
/// format using the PayloadType. The PayloadType indicates what format
/// the data in the Vec<u8> is.
impl TryFrom<(PayloadType, Vec<u8>)> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_from((payload_type, content): (PayloadType, Vec<u8>)) -> Result<Self, Self::Error> {
        Ok(match payload_type {
            PayloadType::Text => PayloadFormat::Text(PayloadFormatText::from(content)),
            PayloadType::Protobuf(options) => PayloadFormat::Protobuf(PayloadFormatProtobuf::new(
                content,
                options.definition(),
                options.message().clone(),
            )?),
            PayloadType::Json => PayloadFormat::Json(PayloadFormatJson::try_from(content)?),
            PayloadType::Yaml => PayloadFormat::Yaml(PayloadFormatYaml::try_from(content)?),
            PayloadType::Hex => PayloadFormat::Hex(PayloadFormatHex::try_from(content)?),
            PayloadType::Base64 => PayloadFormat::Base64(PayloadFormatBase64::try_from(content)?),
            PayloadType::Raw => PayloadFormat::Raw(PayloadFormatRaw::from(content)),
        })
    }
}

impl PayloadFormat {
    pub fn new(
        input_type: &PublishInputType,
        output_type: &PayloadType,
    ) -> Result<PayloadFormat, PayloadFormatError> {
        let content = PayloadFormat::try_from(input_type)?;

        Self::try_from((content, output_type))
    }
}

impl TryFrom<(PayloadFormat, PayloadType)> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_from((payload, target_type): (PayloadFormat, PayloadType)) -> Result<Self, Self::Error> {
        Self::try_from((payload, &target_type))
    }
}

impl TryFrom<&PublishInputType> for PayloadFormat {
    type Error = PayloadFormatError;

    fn try_from(value: &PublishInputType) -> Result<Self, Self::Error> {
        Ok(match value {
            PublishInputType::Text(input) => {
                let c = read_input_type_content_path(input)?;
                PayloadFormat::Text(PayloadFormatText::from(c))
            }
            PublishInputType::Raw(input) => {
                let c = read_from_path(input.path())?;
                PayloadFormat::Raw(PayloadFormatRaw::from(c))
            }
            PublishInputType::Hex(input) => {
                let c = read_input_type_content_path(input)?;
                PayloadFormat::Hex(PayloadFormatHex::try_from(String::from_utf8(c)?)?)
            }
            PublishInputType::Json(input) => {
                let c = read_input_type_content_path(input)?;
                PayloadFormat::Json(PayloadFormatJson::try_from(c)?)
            }
            PublishInputType::Yaml(input) => {
                let c = read_input_type_content_path(input)?;
                PayloadFormat::Yaml(PayloadFormatYaml::try_from(c)?)
            }
            PublishInputType::Base64(input) => {
                let c = read_input_type_content_path(input)?;
                PayloadFormat::Base64(PayloadFormatBase64::try_from(String::from_utf8(c)?)?)
            }
        })
    }
}

fn read_input_type_content_path(
    input: &PublishInputTypeContentPath,
) -> Result<Vec<u8>, PayloadFormatError> {
    if let Some(content) = input.content() {
        Ok(Vec::from(content.as_str()))
    } else if let Some(path) = input.path() {
        read_from_path(path)
    } else {
        return Err(PayloadFormatError::EitherContentOrPathMustBeGiven);
    }
}

fn read_from_path(path: &PathBuf) -> Result<Vec<u8>, PayloadFormatError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return Err(PayloadFormatError::CannotReadInputFromPath(
                e,
                PathBuf::from(path),
            ));
        }
    };

    let mut buf = Vec::new();
    if let Err(e) = file.read_to_end(&mut buf) {
        return Err(PayloadFormatError::CannotReadInputFromPath(
            e,
            PathBuf::from(path),
        ));
    };
    Ok(buf)
}

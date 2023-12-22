use std::fs::read_to_string;
use std::path::PathBuf;

use derive_getters::Getters;
use log::error;
use protofish::context::Context;
use protofish::decode::{MessageValue, Value};

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Debug, Getters)]
pub struct PayloadFormatProtobuf {
    message_value: Box<MessageValue>,
    context: Context,
}

pub struct PayloadFormatProtobufInput {
    content: Vec<u8>,
    definition_file: PathBuf,
    message_name: String,
}

impl PayloadFormatProtobufInput {
    pub fn new(content: Vec<u8>, definition_file: PathBuf, message_name: String) -> Self {
        Self {
            content,
            definition_file,
            message_name,
        }
    }
}

impl TryFrom<PayloadFormatProtobufInput> for PayloadFormatProtobuf {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatProtobufInput) -> Result<Self, Self::Error> {
        let (context, message_value) = get_message_value(
            &value.content,
            &value.definition_file,
            value.message_name.as_str(),
        )?;

        let message_value = Box::new(message_value);
        validate_protobuf(&message_value)?;

        Ok(Self {
            message_value,
            context,
        })
    }
}

impl From<PayloadFormatProtobuf> for Vec<u8> {
    fn from(val: PayloadFormatProtobuf) -> Self {
        val.message_value.encode(&val.context).to_vec()
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatProtobuf {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(_) => Err(Self::Error::ConversionNotPossible(
                String::from("text"),
                String::from("protobuf"),
            )),
            PayloadFormat::Raw(_) => Err(Self::Error::ConversionNotPossible(
                String::from("raw"),
                String::from("protobuf"),
            )),
            PayloadFormat::Protobuf(value) => Ok(value),
            PayloadFormat::Hex(_) => Err(Self::Error::ConversionNotPossible(
                String::from("hex"),
                String::from("protobuf"),
            )),
            PayloadFormat::Base64(_) => Err(Self::Error::ConversionNotPossible(
                String::from("base64"),
                String::from("protobuf"),
            )),
            PayloadFormat::Json(_) => Err(Self::Error::ConversionNotPossible(
                String::from("json"),
                String::from("protobuf"),
            )),
            PayloadFormat::Yaml(_) => Err(Self::Error::ConversionNotPossible(
                String::from("yaml"),
                String::from("protobuf"),
            )),
        }
    }
}

fn validate_protobuf(value: &MessageValue) -> Result<(), PayloadFormatError> {
    for field in &value.fields {
        let result = match &field.value {
            Value::Message(value) => validate_protobuf(value),
            Value::Unknown(_value) => Err(PayloadFormatError::InvalidProtobuf),
            _ => Ok(()),
        };

        result?
    }

    Ok(())
}

fn get_message_value(
    value: &[u8],
    definition_file: &PathBuf,
    message_name: &str,
) -> Result<(Context, MessageValue), PayloadFormatError> {
    let Ok(content) = read_to_string(definition_file) else {
        error!("Could not open definition file {definition_file:?}");
        return Err(PayloadFormatError::CouldNotOpenDefinitionFile(
            definition_file
                .to_str()
                .unwrap_or("invalid path")
                .to_string(),
        ));
    };

    let context = match Context::parse(vec![content]) {
        Ok(context) => context,
        Err(e) => {
            return Err(PayloadFormatError::CouldNotParseProtoFile(e));
        }
    };

    let Some(message_info) = context.get_message(message_name) else {
        return Err(PayloadFormatError::MessageNotFoundInProtoFile(
            message_name.to_owned(),
        ));
    };

    let message_value = message_info.decode(value, &context);
    Ok((context, message_value))
}

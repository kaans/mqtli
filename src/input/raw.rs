use std::fs::read_to_string;
use std::io::Read;
use std::path::PathBuf;

use log::error;
use protofish::context::Context;
use protofish::decode::{MessageValue, Value};

use crate::config::mqtli_config::{PayloadProtobuf, PayloadText, PayloadType, PublishInputTypePath};
use crate::input::{InputError, read_from_path};

pub struct InputConverterRaw {}

impl InputConverterRaw {
    pub fn convert(input: &PublishInputTypePath, output_format: &PayloadType) -> Result<Vec<u8>, InputError> {
        eprintln!("output_format = {:?}", output_format);

        let buf = match read_from_path(input.path()) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };

        match output_format {
            PayloadType::Text(options) => Self::convert_to_text(buf, options),
            PayloadType::Protobuf(options) => Self::convert_to_protobuf(buf, options),
        }
    }

    pub fn convert_to_text(input: Vec<u8>, _options: &PayloadText) -> Result<Vec<u8>, InputError> {
        match String::from_utf8(input) {
            Ok(result) => Ok(result.into_bytes()),
            Err(e) => {
                Err(InputError::CouldNotDecodeUtf8(e))
            }
        }
    }

    fn validate_protobuf(value: Box<MessageValue>) -> Result<(), InputError> {
        for field in value.fields {
            match field.value {
                Value::Message(value) => {
                    Self::validate_protobuf(value)?
                }
                Value::Unknown(_value) => {
                    return Err(InputError::InvalidProtobuf);
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn convert_to_protobuf(input: Vec<u8>, options: &PayloadProtobuf) -> Result<Vec<u8>, InputError> {
        match Self::get_message_value(&input, options.definition(), options.message()) {
            Ok((_context, value)) => {
                Self::validate_protobuf(Box::new(value))?;

                Ok(input)
            }
            Err(e) => Err(e),
        }
    }

    fn get_message_value(value: &Vec<u8>, definition_file: &PathBuf, message_name: &String) -> Result<(Context, MessageValue), InputError> {
        let Ok(content) = read_to_string(definition_file) else {
            error!("Could not open definition file {definition_file:?}");
            return Err(InputError::CouldNotOpenDefinitionFile(definition_file.to_str().unwrap_or("invalid path").to_string()));
        };

        let context = match Context::parse(vec![content]) {
            Ok(context) => context,
            Err(e) => {
                return Err(InputError::CouldNotParseProtoFile(e));
            }
        };

        let Some(message_info) = context.get_message(message_name) else {
            return Err(InputError::MessageNotFoundInProtoFile(message_name.clone()));
        };

        let message_value = message_info.decode(value, &context);
        Ok((context, message_value))
    }
}
use std::fs::read_to_string;
use std::path::PathBuf;

use log::error;
use protofish::context::Context;
use rumqttc::v5::mqttbytes::v5::Publish;

use crate::payload::{OutputFormat, PayloadError};
use crate::payload::protobuf::json_converter::JsonConverter;
use crate::payload::protobuf::plain_converter::PlainConverter;

pub struct PayloadProtobufHandler {}

impl PayloadProtobufHandler {
    pub fn handle_publish(value: &Publish, definition_file: &PathBuf, message_name: &String, output_format: OutputFormat) -> Result<Vec<u8>, PayloadError> {
        let Ok(content) = read_to_string(definition_file) else {
            error!("Could not open definition file {definition_file:?}");
            return Err(PayloadError::CouldNotOpenDefinitionFile(definition_file.to_str().unwrap_or("invalid path").to_string()));
        };

        let context = match Context::parse(vec![content]) {
            Ok(context) => context,
            Err(e) => {
                return Err(PayloadError::CouldNotParseProtoFile(e));
            }
        };

        let Some(message_info) = context.get_message(message_name) else {
            return Err(PayloadError::MessageNotFoundInProtoFile(message_name.clone()));
        };

        let message_value = message_info.decode(value.payload.as_ref(), &context);

        match output_format {
            OutputFormat::PLAIN => {
                PlainConverter::convert(&context, message_value)
            }
            OutputFormat::JSON => {
                JsonConverter::convert(&context, message_value)
            }
            _ => {
                Err(PayloadError::OutputFormatNotSupported(output_format))
            }
        }
    }
}
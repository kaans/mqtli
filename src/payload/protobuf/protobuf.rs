use std::fs::read_to_string;
use std::path::PathBuf;
use base64::Engine;
use base64::engine::general_purpose;
use bytes::Bytes;

use log::error;
use protofish::context::Context;
use rumqttc::v5::mqttbytes::v5::Publish;
use crate::config::mqtli_config::OutputFormat;

use crate::payload::PayloadError;
use crate::payload::protobuf::json_converter::JsonConverter;
use crate::payload::protobuf::plain_converter::PlainConverter;
use crate::payload::protobuf::yaml_converter::YamlConverter;

pub struct PayloadProtobufHandler {}

impl PayloadProtobufHandler {
    pub fn handle_publish(value: &Publish, definition_file: &PathBuf, message_name: &String, output_format: &OutputFormat) -> Result<Vec<u8>, PayloadError> {
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
            OutputFormat::Plain => {
                PlainConverter::convert(&context, message_value)
            }
            OutputFormat::Json => {
                JsonConverter::convert(&context, message_value)
            }
            OutputFormat::Yaml => {
                YamlConverter::convert(&context, message_value)
            }
            OutputFormat::Hex => {
                Self::convert_to_hex(&value.payload)
            }
            OutputFormat::Base64 => {
                Self::convert_to_base64(&value.payload)
            }
        }
    }

    fn convert_to_hex(content: &Bytes) -> Result<Vec<u8>, PayloadError> {
        let hex = hex::encode_upper(content.to_vec());
        Ok(hex.into_bytes())
    }

    fn convert_to_base64(content: &Bytes) -> Result<Vec<u8>, PayloadError> {
        let base64 = general_purpose::STANDARD_NO_PAD.encode(content);
        Ok(base64.into_bytes())
    }
}
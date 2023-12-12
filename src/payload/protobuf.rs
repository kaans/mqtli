use std::fs::read_to_string;
use std::path::PathBuf;

use log::{error, warn};
use protofish::context::{Context, MessageInfo};
use protofish::decode::{FieldValue, MessageValue, Value};
use rumqttc::v5::mqttbytes::v5::Publish;

use crate::payload::{OutputFormat, PayloadError};

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

        let result = match output_format {
            OutputFormat::PLAIN => {
                Self::get_message_value(&context, &Box::new(message_value), 0, None)
            }
            _ => {
                Err(PayloadError::OutputFormatNotSupported(output_format))
            }
        };

        Ok(result?.into_bytes())
    }

    fn get_message_value(context: &Context,
                         message_value: &Box<MessageValue>,
                         indent_level: u16,
                         parent_field: Option<u64>)
                         -> Result<String, PayloadError> {
        let mut result = String::new();

        let message_info = context.resolve_message(message_value.msg_ref);

        let message_text = match parent_field {
            None => {
                format!("{}\n", message_info.full_name)
            }
            Some(parent_field) => {
                let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();
                format!("{indent_spaces}[{}] {}\n", parent_field, message_info.full_name)
            }
        };
        result.push_str(&message_text);

        for field in &message_value.fields {
            let result_field = Self::get_field_value(context, message_info, &field, indent_level + 1)?;
            result.push_str(&result_field);
        }

        Ok(result)
    }

    fn get_field_value(context: &Context, message_response: &MessageInfo, field_value: &FieldValue, indent_level: u16) -> Result<String, PayloadError> {
        let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();

        return match &message_response.get_field(field_value.number) {
            None => {
                Err(PayloadError::FieldNumberNotFoundInProtoFile(field_value.number))
            }
            Some(field) => {
                let type_name = &field.name;

                let ret = match &field_value.value {
                    Value::Double(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Double)\n", field.number, value.to_string())
                    }
                    Value::Float(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Float)\n", field.number, value.to_string())
                    }
                    Value::Int32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Int32)\n", field.number, value.to_string())
                    }
                    Value::Int64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Int64)\n", field.number, value.to_string())
                    }
                    Value::UInt32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (UInt32)\n", field.number, value.to_string())
                    }
                    Value::UInt64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (UInt64)\n", field.number, value.to_string())
                    }
                    Value::SInt32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SInt32)\n", field.number, value.to_string())
                    }
                    Value::SInt64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SInt64)\n", field.number, value.to_string())
                    }
                    Value::Fixed32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Fixed32)\n", field.number, value.to_string())
                    }
                    Value::Fixed64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Fixed64)\n", field.number, value.to_string())
                    }
                    Value::SFixed32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SFixed32)\n", field.number, value.to_string())
                    }
                    Value::SFixed64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SFixed64)\n", field.number, value.to_string())
                    }
                    Value::Bool(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Bool)\n", field.number, value.to_string())
                    }
                    Value::String(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (String)\n", field.number, value)
                    }
                    Value::Bytes(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {:?} (Bytes)\n", field.number, value)
                    }
                    Value::Message(value) => {
                        Self::get_message_value(context, value, indent_level, Some(field.number))?
                    }
                    value => {
                        format!("{indent_spaces}[{}] Unknown value encountered: {:?}\n", field.number, value)
                    }
                };

                Ok(ret)
            }
        };
    }
}
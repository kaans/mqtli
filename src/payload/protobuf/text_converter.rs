use protofish::context::{Context, MessageInfo};
use protofish::decode::{FieldValue, MessageValue, Value};

use crate::payload::PayloadError;

pub struct TextConverter {}

impl TextConverter {
    pub fn convert(context: &Context,
                   message_value: MessageValue)
                   -> Result<Vec<u8>, PayloadError> {

        let result = Self::get_message_value(context, &Box::new(message_value), 0, None)?;

        Ok(result.into_bytes())
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
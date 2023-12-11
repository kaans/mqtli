use std::fs::read_to_string;
use std::path::PathBuf;

use log::{error, warn};
use protofish::context::{Context, MessageInfo};
use protofish::decode::{FieldValue, Value};
use rumqttc::v5::mqttbytes::v5::Publish;

pub struct PayloadProtobufHandler {}

impl PayloadProtobufHandler {
    pub fn handle_publish(value: &Publish, definition_file: &PathBuf, message_name: &String) {
        let Ok(content) = read_to_string(definition_file) else {
            error!("Could not open definition file {definition_file:?}");
            return;
        };

        let context = match Context::parse(vec![content]) {
            Ok(context) => context,
            Err(e) => {
                error!("Could not parse proto file: {e:?}");
                return;
            }
        };

        let Some(message_response) = context.get_message(message_name) else {
            error!("Message \"{}\" not found in proto file, cannot decode payload", message_name);
            return;
        };

        let message_value = message_response.decode(value.payload.as_ref(), &context);

        for field in message_value.fields {
            if let Ok(result) = Self::get_field_value(&context, &message_response, &field, 0) {
                println!("{}", result);
            }
        }
    }

    fn get_field_value(context: &Context, message_response: &MessageInfo, field_value: &FieldValue, indent_level: u16) -> Result<String, ()> {
        let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();

        return match &message_response.get_field(field_value.number) {
            None => {
                error!("Field with number {} not found in message", field_value.number);

                Err(())
            }
            Some(field) => {
                let type_name = &field.name;

                let ret = match &field_value.value {
                    Value::Double(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Double)", field.number, value.to_string())
                    }
                    Value::Float(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Float)", field.number, value.to_string())
                    }
                    Value::Int32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Int32)", field.number, value.to_string())
                    }
                    Value::Int64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Int64)", field.number, value.to_string())
                    }
                    Value::UInt32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (UInt32)", field.number, value.to_string())
                    }
                    Value::UInt64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (UInt64)", field.number, value.to_string())
                    }
                    Value::SInt32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SInt32)", field.number, value.to_string())
                    }
                    Value::SInt64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SInt64)", field.number, value.to_string())
                    }
                    Value::Fixed32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Fixed32)", field.number, value.to_string())
                    }
                    Value::Fixed64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Fixed64)", field.number, value.to_string())
                    }
                    Value::SFixed32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SFixed32)", field.number, value.to_string())
                    }
                    Value::SFixed64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SFixed64)", field.number, value.to_string())
                    }
                    Value::Bool(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Bool)", field.number, value.to_string())
                    }
                    Value::String(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (String)", field.number, value)
                    }
                    Value::Bytes(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {:?} (Bytes)", field.number, value)
                    }
                    Value::Message(value) => {
                        let message_inner = context.resolve_message(value.msg_ref);
                        let mut ret = format!("[{}] {}", field.number, &message_inner.full_name);

                        for field in &value.fields {
                            let ret_inner = Self::get_field_value(context, message_inner, &field, indent_level + 1)?;

                            ret.push_str(format!("\n{}", ret_inner).as_str());
                        }

                        ret
                    }
                    value => {
                        warn!("Unknown value encountered: {:?}", value);

                        "Unknown".to_string()
                    }
                };

                Ok(ret)
            }
        };
    }
}
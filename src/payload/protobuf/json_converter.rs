use protofish::context::Context;
use protofish::decode::{FieldValue, MessageValue, Value};
use serde_json::json;

use crate::payload::PayloadError;

pub struct JsonConverter {}

impl JsonConverter {
    pub fn convert(context: &Context,
                   message_value: MessageValue)
                   -> Result<Vec<u8>, PayloadError> {
        let result = Self::get_message_value(context, &Box::new(message_value))?;

        Ok(result.to_string().into_bytes())
    }

    fn get_message_value(context: &Context,
                         message_value: &Box<MessageValue>)
                         -> Result<serde_json::Value, PayloadError> {
        let message_info = context.resolve_message(message_value.msg_ref);

        let mut map_fields = serde_json::Map::new();

        for field in &message_value.fields {
            let result_field = Self::get_field_value(context, &field)?;
            let field_name = match &message_info.get_field(field.number) {
                None => "unknown",
                Some(value) => value.name.as_str()
            };
            map_fields.insert(field_name.to_string(), result_field);
        }

        Ok(serde_json::Value::Object(map_fields))
    }

    fn get_field_value(
        context: &Context,
        field_value: &FieldValue)
        -> Result<serde_json::Value, PayloadError> {
        let result = match &field_value.value {
            Value::Double(value) => {
                json!(value)
            }
            Value::Float(value) => {
                json!(value)
            }
            Value::Int32(value) => {
                json!(value)
            }
            Value::Int64(value) => {
                json!(value)
            }
            Value::UInt32(value) => {
                json!(value)
            }
            Value::UInt64(value) => {
                json!(value)
            }
            Value::SInt32(value) => {
                json!(value)
            }
            Value::SInt64(value) => {
                json!(value)
            }
            Value::Fixed32(value) => {
                json!(value)
            }
            Value::Fixed64(value) => {
                json!(value)
            }
            Value::SFixed32(value) => {
                json!(value)
            }
            Value::SFixed64(value) => {
                json!(value)
            }
            Value::Bool(value) => {
                json!(value)
            }
            Value::String(value) => {
                json!(value)
            }
            Value::Message(value) => {
                Self::get_message_value(context, value)?
            }
            value => {
                json!(format!("Unknown: {:?}", value))
            }
        };

        Ok(result)
    }
}
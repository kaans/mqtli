use protofish::context::Context;
use protofish::decode::{FieldValue, MessageValue, Value};
use serde_yaml::value;

use crate::payload::PayloadError;

pub struct YamlConverter {}

impl YamlConverter {
    pub fn convert(context: &Context,
                   message_value: MessageValue)
                   -> Result<Vec<u8>, PayloadError> {
        let result = Self::get_message_value(context, &Box::new(message_value))?;

        match serde_yaml::to_string(&result) {
            Ok(yaml) => Ok(yaml.into_bytes()),
            Err(e) => {
                Err(PayloadError::CouldNotConvertToYaml(e))
            }
        }
    }

    fn get_message_value(context: &Context,
                         message_value: &Box<MessageValue>)
                         -> Result<serde_yaml::Value, PayloadError> {
        let message_info = context.resolve_message(message_value.msg_ref);

        let mut map_fields = serde_yaml::Mapping::new();

        for field in &message_value.fields {
            let result_field = Self::get_field_value(context, &field)?;
            let field_name = match &message_info.get_field(field.number) {
                None => "unknown",
                Some(value) => value.name.as_str()
            };
            map_fields.insert(serde_yaml::Value::String(field_name.to_string()), result_field);
        }

        Ok(serde_yaml::Value::Mapping(map_fields))
    }

    fn get_field_value(
        context: &Context,
        field_value: &FieldValue)
        -> Result<serde_yaml::Value, PayloadError> {
        let result = match &field_value.value {
            Value::Double(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::Float(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::Int32(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::Int64(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::UInt32(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::UInt64(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::SInt32(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::SInt64(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::Fixed32(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::Fixed64(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::SFixed32(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::SFixed64(value) => {
                serde_yaml::Value::Number(value::Number::from(*value))
            }
            Value::Bool(value) => {
                serde_yaml::Value::Bool(*value)
            }
            Value::String(value) => {
                serde_yaml::Value::String(value.clone())
            }
            Value::Message(value) => {
                Self::get_message_value(context, value)?
            }
            value => {
                serde_yaml::Value::String(format!("Unknown: {:?}", value))
            }
        };

        Ok(result)
    }
}
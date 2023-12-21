use serde_json::{from_slice, Value};

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatJson {
    content: Value,
}

pub type PayloadFormatJsonInput = Vec<u8>;

impl TryFrom<PayloadFormatJsonInput> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatJsonInput) -> Result<Self, Self::Error> {
        let content = from_slice(value.as_slice())?;

        Ok(Self {
            content
        })
    }
}

impl Into<Vec<u8>> for PayloadFormatJson {
    fn into(self) -> Vec<u8> {
        self.content.to_string().into_bytes()
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Raw(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Protobuf(value) => {
                Ok(Self {
                    content: protobuf::get_message_value(value.context(), value.message_value())?
                })
            }
            PayloadFormat::Hex(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Base64(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Json(value) => Ok(value),
            PayloadFormat::Yaml(value) => {
                let a: Vec<u8> = value.try_into()?;
                Self::try_from(a)
            }
        }
    }
}

mod protobuf {
    use protofish::context::Context;
    use protofish::decode::{FieldValue, MessageValue, Value};
    use serde_json::json;
    use crate::payload::PayloadFormatError;

    pub(super) fn get_message_value(context: &Context,
                                    message_value: &Box<MessageValue>)
                                    -> Result<serde_json::Value, PayloadFormatError> {
        let message_info = context.resolve_message(message_value.msg_ref);

        let mut map_fields = serde_json::Map::new();

        for field in &message_value.fields {
            let result_field = get_field_value(context, &field)?;
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
        -> Result<serde_json::Value, PayloadFormatError> {
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
                get_message_value(context, value)?
            }
            value => {
                json!(format!("Unknown: {:?}", value))
            }
        };

        Ok(result)
    }
}
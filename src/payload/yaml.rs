use serde_yaml::{from_slice, Value};

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatYaml {
    content: Value,
}

pub type PayloadFormatYamlInput = Vec<u8>;

impl TryFrom<PayloadFormatYamlInput> for PayloadFormatYaml {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatYamlInput) -> Result<Self, Self::Error> {
        let content = from_slice(value.as_slice())?;

        Ok(Self { content })
    }
}

impl TryFrom<PayloadFormatYaml> for Vec<u8> {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatYaml) -> Result<Self, Self::Error> {
        Ok(serde_yaml::to_string(&value.content)?.into_bytes())
    }
}

impl TryFrom<PayloadFormatYaml> for String {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatYaml) -> Result<Self, Self::Error> {
        let result: Result<Vec<u8>, Self::Error> = value.try_into();
        Ok(String::from_utf8(result?)?)
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatYaml {
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
            PayloadFormat::Protobuf(value) => Ok(Self {
                content: protobuf::get_message_value(value.context(), value.message_value())?,
            }),
            PayloadFormat::Hex(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Base64(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Yaml(value) => Ok(value),
            PayloadFormat::Json(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
        }
    }
}

mod protobuf {
    use protofish::context::Context;
    use protofish::decode::{FieldValue, MessageValue, Value};
    use serde_yaml::value;

    use crate::payload::PayloadFormatError;

    pub(super) fn get_message_value(
        context: &Context,
        message_value: &MessageValue,
    ) -> Result<serde_yaml::Value, PayloadFormatError> {
        let message_info = context.resolve_message(message_value.msg_ref);

        let mut map_fields = serde_yaml::Mapping::new();

        for field in &message_value.fields {
            let result_field = get_field_value(context, field)?;
            let field_name = match &message_info.get_field(field.number) {
                None => "unknown",
                Some(value) => value.name.as_str(),
            };
            map_fields.insert(
                serde_yaml::Value::String(field_name.to_string()),
                result_field,
            );
        }

        Ok(serde_yaml::Value::Mapping(map_fields))
    }

    fn get_field_value(
        context: &Context,
        field_value: &FieldValue,
    ) -> Result<serde_yaml::Value, PayloadFormatError> {
        let result = match &field_value.value {
            Value::Double(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::Float(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::Int32(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::Int64(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::UInt32(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::UInt64(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::SInt32(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::SInt64(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::Fixed32(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::Fixed64(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::SFixed32(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::SFixed64(value) => serde_yaml::Value::Number(value::Number::from(*value)),
            Value::Bool(value) => serde_yaml::Value::Bool(*value),
            Value::String(value) => serde_yaml::Value::String(value.clone()),
            Value::Message(value) => get_message_value(context, value)?,
            value => serde_yaml::Value::String(format!("Unknown: {:?}", value)),
        };

        Ok(result)
    }
}

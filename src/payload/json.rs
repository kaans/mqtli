use derive_getters::Getters;
use serde_json::{from_slice, from_str, Value};

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug, Getters)]
pub struct PayloadFormatJson {
    content: Value,
}

impl TryFrom<Vec<u8>> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let content = from_slice(value.as_slice())?;

        Ok(Self { content })
    }
}

impl TryFrom<String> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let content = from_slice(value.as_bytes())?;

        Ok(Self { content })
    }
}

impl From<Value> for PayloadFormatJson {
    fn from(val: Value) -> Self {
        Self { content: val }
    }
}

impl From<PayloadFormatJson> for Vec<u8> {
    fn from(val: PayloadFormatJson) -> Self {
        val.content.to_string().into_bytes()
    }
}

impl From<PayloadFormatJson> for String {
    fn from(val: PayloadFormatJson) -> Self {
        val.content.to_string()
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => {
                Self::convert_from_value(value.into())
            }
            PayloadFormat::Raw(_value) => {
                Err(PayloadFormatError::ConversionNotPossible("raw".to_string(), "json".to_string()))
            }
            PayloadFormat::Protobuf(value) => Ok(Self {
                content: protobuf::get_message_value(value.context(), value.message_value())?,
            }),
            PayloadFormat::Hex(value) => {
                Self::convert_from_value(value.into())
            }
            PayloadFormat::Base64(value) => {
                Self::convert_from_value(value.into())
            }
            PayloadFormat::Json(value) => Ok(value),
            PayloadFormat::Yaml(value) => {
                let Some(text_node) = value.content().get("content") else {
                    return Err(PayloadFormatError::CouldNotConvertFromYaml(
                        "Attribute \"content\" not found".to_string(),
                    ));
                };

                let Some(text_node) = text_node.as_str() else {
                    return Err(PayloadFormatError::CouldNotConvertFromYaml(
                        "Could not read attribute \"content\" as string".to_string(),
                    ));
                };

                Self::convert_from_value(text_node.to_string())
            }
        }
    }
}

impl PayloadFormatJson {
    fn convert_from_value(value: String) -> Result<PayloadFormatJson, PayloadFormatError> {
        let v: Value = from_str(format!("{{\"content\": \"{}\"}}", value).as_str())?;
        Ok(PayloadFormatJson::from(v))
    }
}

mod protobuf {
    use protofish::context::Context;
    use protofish::decode::{FieldValue, MessageValue, Value};
    use serde_json::json;

    use crate::payload::PayloadFormatError;

    pub(super) fn get_message_value(
        context: &Context,
        message_value: &MessageValue,
    ) -> Result<serde_json::Value, PayloadFormatError> {
        let message_info = context.resolve_message(message_value.msg_ref);

        let mut map_fields = serde_json::Map::new();

        for field in &message_value.fields {
            let result_field = get_field_value(context, field)?;
            let field_name = match &message_info.get_field(field.number) {
                None => "unknown",
                Some(value) => value.name.as_str(),
            };
            map_fields.insert(field_name.to_string(), result_field);
        }

        Ok(serde_json::Value::Object(map_fields))
    }

    fn get_field_value(
        context: &Context,
        field_value: &FieldValue,
    ) -> Result<serde_json::Value, PayloadFormatError> {
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
            Value::Message(value) => get_message_value(context, value)?,
            value => {
                json!(format!("Unknown: {:?}", value))
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::text::PayloadFormatText;
    use crate::payload::yaml::PayloadFormatYaml;

    use super::*;

    const INPUT_STRING: &str = "INPUT";
    const INPUT_STRING_HEX: &str = "494e505554";
    // INPUT
    const INPUT_STRING_BASE64: &str = "SU5QVVQ="; // INPUT

    fn get_input() -> Vec<u8> {
        get_input_json(INPUT_STRING).into()
    }

    fn get_input_json(value: &str) -> String {
        format!("{{\"content\":\"{}\"}}", value)
    }
    fn get_json_value(value: &str) -> Value {
        from_str(get_input_json(value).as_str()).unwrap()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatJson::try_from(get_input()).unwrap();

        assert_eq!(get_json_value(INPUT_STRING), result.content);
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatJson::try_from(get_input()).unwrap();

        let result: Vec<u8> = input.into();
        assert_eq!(get_input_json(INPUT_STRING).as_bytes(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatJson::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::from(input);
        assert_eq!(get_input_json(INPUT_STRING).as_bytes(), result.as_slice());
    }

    #[test]
    fn to_string_into() {
        let input = PayloadFormatJson::try_from(get_input()).unwrap();

        let result: String = input.into();
        assert_eq!(get_input_json(INPUT_STRING), result.as_str());
    }

    #[test]
    fn to_string_from() {
        let input = PayloadFormatJson::try_from(get_input()).unwrap();

        let result: String = String::from(input);
        assert_eq!(get_input_json(INPUT_STRING), result.as_str());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::from(INPUT_STRING.to_string());
        let result = PayloadFormatJson::try_from(PayloadFormat::Text(input)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING), result.content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Raw(input));

        assert!(result.is_err());
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING_HEX), result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING_BASE64), result.content);
    }

    #[test]
    fn from_json() {
        let input =
            PayloadFormatJson::try_from(Vec::<u8>::from(format!("{{\"content\": \"{}\"}}", INPUT_STRING))).unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING), result.content);
    }

    #[test]
    fn from_yaml() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(format!("content: \"{}\"", INPUT_STRING))).unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING), result.content);
    }
}

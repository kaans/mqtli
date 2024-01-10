use std::fmt::{Display, Formatter};

use derive_getters::Getters;
use serde_json::{from_slice, Value};

use crate::config::PayloadJson;
use crate::payload::{convert_raw_type, PayloadFormat, PayloadFormatError};

/// This payload format contains a JSON payload. Its value is encoded as
/// `serde_json::Value`.
///
/// If this payload is constructed from a format which cannot be converted
/// to JSON, a JSON object is constructed with one field `content` which holds
/// the value: `{ "content": "..." }`
#[derive(Clone, Debug, Default, Getters)]
pub struct PayloadFormatJson {
    content: Value,
}

impl PayloadFormatJson {
    fn decode_from_json_payload(value: &PayloadFormatJson) -> String {
        value.content.to_string()
    }

    fn encode_to_json(value: Vec<u8>) -> serde_json::Result<Value> {
        from_slice(value.as_slice())
    }
}

/// Displays the hex encoded content.
impl Display for PayloadFormatJson {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// Decode JSON payload format from a `Vec<u8>`.
///
/// The `Vec<u8>` must contain a valid JSON string.
///
/// # Examples
///
/// ```
/// let input = "{\"content\":\"INPUT\"}";
///
/// let payload_json = PayloadFormatJson::try_from(Vec::from(input)).unwrap();
///
/// assert_eq!("INPUT", result.content.get("content").unwrap().as_str());
/// ```
impl TryFrom<Vec<u8>> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self {
            content: Self::encode_to_json(value)?,
        })
    }
}

/// Decode JSON payload format from a `String`.
///
/// The `String` must contain a valid JSON string.
///
/// # Examples
///
/// ```
/// let input = String::from("{\"content\":\"INPUT\"}");
///
/// let payload_json = PayloadFormatJson::try_from(input).unwrap();
///
/// assert_eq!("INPUT", result.content.get("content").unwrap().as_str());
/// ```
impl TryFrom<String> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.into_bytes())
    }
}

/// Decode JSON payload format from a `serde_json::Value`.
///
/// # Examples
///
/// ```
/// let input = json!({ "content": "INPUT" });
///
/// let payload_json = PayloadFormatJson::from(input);
///
/// assert_eq!("INPUT", result.content.get("content").unwrap().as_str());
/// ```
impl From<Value> for PayloadFormatJson {
    fn from(val: Value) -> Self {
        Self { content: val }
    }
}

/// Decode the content of a json payload format to `Vec<u8>`.
///
/// The resulting `Vec<u8>` contains a valid UTF-8 encoded JSON string.
///
/// # Examples
///
/// ```
/// let input = json!({ "content": "INPUT" });
///
/// let result: Vec<u8> = Vec::from(input);
///
/// assert_eq!("{\"content\":\"INPUT\"}", String::from(result));
/// ```
impl From<PayloadFormatJson> for Vec<u8> {
    fn from(val: PayloadFormatJson) -> Self {
        <PayloadFormatJson as Into<String>>::into(val).into_bytes()
    }
}

/// Decode the content of a json payload format to `String`.
///
/// The resulting `String` contains a valid UTF-8 encoded JSON string.
///
/// # Examples
///
/// ```
/// let input = json!({ "content": "INPUT" });
///
/// let result: String = Vec::from(input);
///
/// assert_eq!("{\"content\":\"INPUT\"}", String::from(result));
/// ```
impl From<PayloadFormatJson> for String {
    fn from(val: PayloadFormatJson) -> Self {
        PayloadFormatJson::decode_from_json_payload(&val)
    }
}

/// Decode JSON payload format from a another `PayloadFormat`.
///
/// The resulting JSON value depends on the type of the `PayloadFormat`:
///
/// | `PayloadFormat` | Result |
/// |---------|---------|
/// | Text     | `{ "content": "..." }` |
/// | Raw     | conversion not possible (JSON only accepts UTF-8/16/32, raw can be anything) |
/// | Protobuf     | The structure of the protobuf message as JSON Object |
/// | Hex     | `{ "content": "..." }` |
/// | Base64     | `{ "content": "..." }` |
/// | JSON     | the incoming value itself (no conversion is done) |
/// | YAML     | The structure of the YAML input as JSON |
///
/// # Examples
///
/// ```
/// let input = PayloadFormatText::from("INPUT");
///
/// let payload_json = PayloadFormatJson::try_from(input).unwrap();
///
/// assert_eq!("INPUT", result.content.get("content").unwrap().as_str());
/// ```
impl TryFrom<PayloadFormat> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        Self::try_from((value, &PayloadJson::default()))
    }
}

impl TryFrom<(PayloadFormat, &PayloadJson)> for PayloadFormatJson {
    type Error = PayloadFormatError;

    fn try_from((value, options): (PayloadFormat, &PayloadJson)) -> Result<Self, Self::Error> {
        fn encode_to_json_with_string_content(
            value: String,
        ) -> Result<PayloadFormatJson, PayloadFormatError> {
            Ok(PayloadFormatJson::from(PayloadFormatJson::encode_to_json(
                format!("{{\"content\": \"{}\"}}", value).into_bytes(),
            )?))
        }

        match value {
            PayloadFormat::Text(value) => encode_to_json_with_string_content(value.into()),
            PayloadFormat::Raw(value) => encode_to_json_with_string_content(
                convert_raw_type(options.raw_as_type(), value.into()),
            ),
            PayloadFormat::Protobuf(value) => Ok(Self {
                content: protobuf::get_message_value(
                    value.context(),
                    value.message_value(),
                    options,
                )?,
            }),
            PayloadFormat::Hex(value) => encode_to_json_with_string_content(value.into()),
            PayloadFormat::Base64(value) => encode_to_json_with_string_content(value.into()),
            PayloadFormat::Json(value) => Ok(value),
            PayloadFormat::Yaml(value) => Ok(Self {
                content: serde_yaml::from_value::<Value>(value.content().clone())?,
            }),
        }
    }
}

mod protobuf {
    use protofish::context::Context;
    use protofish::decode::{FieldValue, MessageValue, Value};
    use serde_json::json;

    use crate::config::PayloadJson;
    use crate::payload::{convert_raw_type, PayloadFormatError};

    pub(super) fn get_message_value(
        context: &Context,
        message_value: &MessageValue,
        options: &PayloadJson,
    ) -> Result<serde_json::Value, PayloadFormatError> {
        let message_info = context.resolve_message(message_value.msg_ref);

        let mut map_fields = serde_json::Map::new();

        for field in &message_value.fields {
            let result_field = get_field_value(context, field, options)?;
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
        options: &PayloadJson,
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
            Value::Message(value) => get_message_value(context, value, options)?,
            Value::Bytes(value) => {
                json!(convert_raw_type(options.raw_as_type(), value.to_vec()))
            }
            Value::Enum(value) => {
                let enum_ref = value.enum_ref;
                let enum_value = match context
                    .resolve_enum(enum_ref)
                    .get_field_by_value(value.value)
                {
                    None => "Unknown".to_string(),
                    Some(value) => value.name.clone(),
                };

                json!(enum_value)
            }
            value => {
                // TODO implement missing protobuf values for json conversion
                json!(format!("Unknown: {:?}", value))
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;
    use crate::config::PayloadOptionRawFormat;

    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::protobuf::PayloadFormatProtobuf;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::text::PayloadFormatText;
    use crate::payload::yaml::PayloadFormatYaml;

    use super::*;

    const INPUT_STRING: &str = "INPUT";
    const INPUT_STRING_HEX: &str = "494e505554";
    const INPUT_STRING_BASE64: &str = "SU5QVVQ=";
    const INPUT_STRING_PROTOBUF_AS_HEX: &str = "082012080a066b696e646f6618012202c328";
    const INPUT_STRING_MESSAGE: &str = r#"
    syntax = "proto3";
    package Proto;

    enum Position {
        POSITION_UNSPECIFIED = 0;
        POSITION_INSIDE = 1;
        POSITION_OUTSIDE = 2;
    }

    message Inner { string kind = 1; }

    message Response {
      int32 distance = 1;
      Inner inside = 2;
      Position position = 3;
      bytes raw = 4;
    }
    "#;
    const MESSAGE_NAME: &str = "Proto.Response";

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
    fn from_raw_as_hex() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadJson::default();
        let result = PayloadFormatJson::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING_HEX), result.content);
    }

    #[test]
    fn from_raw_as_base64() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadJson::new(PayloadOptionRawFormat::Base64);
        let result = PayloadFormatJson::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING_BASE64), result.content);
    }

    #[test]
    fn from_raw_as_utf8() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadJson::new(PayloadOptionRawFormat::Utf8);
        let result = PayloadFormatJson::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING), result.content);
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
            PayloadFormatJson::try_from(Vec::<u8>::from(get_input_json(INPUT_STRING))).unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(get_json_value(INPUT_STRING), result.content);
    }

    #[test]
    fn from_yaml() {
        let input =
            PayloadFormatYaml::try_from(Vec::<u8>::from(format!("content: \"{}\"", INPUT_STRING)))
                .unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(
            INPUT_STRING,
            result.content.get("content").unwrap().as_str().unwrap()
        );
    }

    #[test]
    fn from_yaml_complex() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(format!(
            "size: 54.3\nname: \"{}\"",
            INPUT_STRING
        )))
            .unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(54.3, result.content.get("size").unwrap().as_f64().unwrap());
        assert_eq!(
            INPUT_STRING,
            result.content.get("name").unwrap().as_str().unwrap()
        );
    }

    #[test]
    fn from_yaml_array() {
        let input_string = r#"
            size: 54.3
            name: full name
            colors:
              - red
              - blue
              - green
        "#;

        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(input_string)).unwrap();
        let result = PayloadFormatJson::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(54.3, result.content.get("size").unwrap().as_f64().unwrap());
        assert_eq!(
            "full name",
            result.content.get("name").unwrap().as_str().unwrap()
        );
        assert_eq!(
            "red",
            result
                .content
                .get("colors")
                .unwrap()
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_str()
                .unwrap()
        );
        assert_eq!(
            "blue",
            result
                .content
                .get("colors")
                .unwrap()
                .as_array()
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .unwrap()
        );
        assert_eq!(
            "green",
            result
                .content
                .get("colors")
                .unwrap()
                .as_array()
                .unwrap()
                .get(2)
                .unwrap()
                .as_str()
                .unwrap()
        );
    }

    #[test]
    fn from_protobuf() {
        let input = PayloadFormatProtobuf::new(
            hex::decode(INPUT_STRING_PROTOBUF_AS_HEX).unwrap(),
            String::from(INPUT_STRING_MESSAGE),
            MESSAGE_NAME.to_string(),
        );
        let result = PayloadFormatJson::try_from(PayloadFormat::Protobuf(input.unwrap())).unwrap();

        assert_eq!(
            32,
            result.content.get("distance").unwrap().as_i64().unwrap()
        );
        assert_eq!(
            "POSITION_INSIDE",
            result.content.get("position").unwrap().as_str().unwrap()
        );
        assert_eq!(
            "kindof",
            result
                .content
                .get("inside")
                .unwrap()
                .as_object()
                .unwrap()
                .get("kind")
                .unwrap()
                .as_str()
                .unwrap()
        );
    }

    #[test]
    fn from_protobuf_raw_as_hex() {
        let input = PayloadFormatProtobuf::new(
            hex::decode(INPUT_STRING_PROTOBUF_AS_HEX).unwrap(),
            String::from(INPUT_STRING_MESSAGE),
            MESSAGE_NAME.to_string(),
        );
        let options = PayloadJson::default();
        let result =
            PayloadFormatJson::try_from((PayloadFormat::Protobuf(input.unwrap()), &options))
                .unwrap();

        assert_eq!(
            "c328".to_string(),
            result.content.get("raw").unwrap().as_str().unwrap()
        );
    }

    #[test]
    fn from_protobuf_raw_as_base64() {
        let input = PayloadFormatProtobuf::new(
            hex::decode(INPUT_STRING_PROTOBUF_AS_HEX).unwrap(),
            String::from(INPUT_STRING_MESSAGE),
            MESSAGE_NAME.to_string(),
        );
        let options = PayloadJson::new(PayloadOptionRawFormat::Base64);
        let result =
            PayloadFormatJson::try_from((PayloadFormat::Protobuf(input.unwrap()), &options))
                .unwrap();

        assert_eq!(
            "wyg=".to_string(),
            result.content.get("raw").unwrap().as_str().unwrap()
        );
    }
}

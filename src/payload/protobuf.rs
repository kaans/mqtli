use std::fs::read_to_string;
use std::path::PathBuf;

use derive_getters::Getters;
use log::error;
use protofish::context::Context;
use protofish::decode::{MessageValue, Value};

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Debug, Getters)]
pub struct PayloadFormatProtobuf {
    message_value: Box<MessageValue>,
    context: Context,
}

impl PayloadFormatProtobuf {
    pub fn new(content: Vec<u8>, definition_file_content: String, message_name: String) -> Result<Self, PayloadFormatError> {
        let (context, message_value) =
            get_message_value(&content, definition_file_content, message_name.as_str())?;

        let message_value = Box::new(message_value);
        validate_protobuf(&message_value)?;

        Ok(Self {
            message_value,
            context,
        })
    }

    pub fn new_from_definition_file(content: Vec<u8>, definition_file: &PathBuf, message_name: String) -> Result<Self, PayloadFormatError> {
        let content_from_file = match Self::read_definition_file(definition_file) {
            Ok(value) => value,
            Err(value) => return value,
        };

        PayloadFormatProtobuf::new(content, content_from_file.to_string(), message_name)
    }

    pub fn convert_from(payload: PayloadFormat, definition_file_content: String, message_name: &str) -> Result<Self, PayloadFormatError> {
        let content: Vec<u8> = match payload {
            PayloadFormat::Text(_value) => return Err(PayloadFormatError::ConversionNotPossible("text".to_string(), "protobuf".to_string())),
            PayloadFormat::Raw(value) => Vec::from(value),
            PayloadFormat::Protobuf(value) => Vec::from(value),
            PayloadFormat::Hex(value) => Vec::from(value),
            PayloadFormat::Base64(value) => Vec::from(value),
            PayloadFormat::Json(_value) => return Err(PayloadFormatError::ConversionNotPossible("text".to_string(), "protobuf".to_string())),
            PayloadFormat::Yaml(_value) => return Err(PayloadFormatError::ConversionNotPossible("text".to_string(), "protobuf".to_string())),
        };

        let (context, message_value) =
            get_message_value(&content, definition_file_content, message_name)?;

        let message_value = Box::new(message_value);
        validate_protobuf(&message_value)?;

        Ok(Self {
            message_value,
            context,
        })
    }

    pub fn convert_from_definition_file(payload: PayloadFormat, definition_file: &PathBuf, message_name: &str) -> Result<Self, PayloadFormatError> {
        let definition_file_content = match Self::read_definition_file(definition_file) {
            Ok(value) => value,
            Err(value) => return value,
        };

        Self::convert_from(payload, definition_file_content, message_name)
    }

    fn read_definition_file(definition_file: &PathBuf) -> Result<String, Result<PayloadFormatProtobuf, PayloadFormatError>> {
        let Ok(content_from_file) = read_to_string(definition_file) else {
            error!("Could not open definition file {definition_file:?}");
            return Err(Err(PayloadFormatError::CouldNotOpenDefinitionFile(
                definition_file
                    .to_str()
                    .unwrap_or("invalid path")
                    .to_string(),
            )));
        };
        Ok(content_from_file)
    }
}

impl From<PayloadFormatProtobuf> for Vec<u8> {
    fn from(val: PayloadFormatProtobuf) -> Self {
        val.message_value.encode(&val.context).to_vec()
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatProtobuf {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(_) => Err(Self::Error::ConversionNotPossible(
                String::from("text"),
                String::from("protobuf"),
            )),
            PayloadFormat::Raw(_) => Err(Self::Error::ConversionNotPossible(
                String::from("raw"),
                String::from("protobuf"),
            )),
            PayloadFormat::Protobuf(value) => Ok(value),
            PayloadFormat::Hex(_) => Err(Self::Error::ConversionNotPossible(
                String::from("hex"),
                String::from("protobuf"),
            )),
            PayloadFormat::Base64(_) => Err(Self::Error::ConversionNotPossible(
                String::from("base64"),
                String::from("protobuf"),
            )),
            PayloadFormat::Json(_) => Err(Self::Error::ConversionNotPossible(
                String::from("json"),
                String::from("protobuf"),
            )),
            PayloadFormat::Yaml(_) => Err(Self::Error::ConversionNotPossible(
                String::from("yaml"),
                String::from("protobuf"),
            )),
        }
    }
}

fn validate_protobuf(value: &MessageValue) -> Result<(), PayloadFormatError> {
    for field in &value.fields {
        let result = match &field.value {
            Value::Message(value) => validate_protobuf(value),
            Value::Unknown(_value) => Err(PayloadFormatError::InvalidProtobuf),
            _ => Ok(()),
        };

        result?
    }

    Ok(())
}

fn get_message_value(
    value: &[u8],
    content: String,
    message_name: &str,
) -> Result<(Context, MessageValue), PayloadFormatError> {
    let context = match Context::parse(vec![content.clone()]) {
        Ok(context) => context,
        Err(e) => {
            return Err(PayloadFormatError::CouldNotParseProtoFile(e));
        }
    };

    let Some(message_info) = context.get_message(message_name) else {
        return Err(PayloadFormatError::MessageNotFoundInProtoFile(
            message_name.to_owned(),
        ));
    };

    let message_value = message_info.decode(value, &context);
    Ok((context, message_value))
}

#[cfg(test)]
mod tests {
    use protofish::decode::FieldValue;

    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::text::PayloadFormatText;
    use crate::payload::yaml::PayloadFormatYaml;

    use super::*;

    const INPUT_STRING_JSON: &str = r#"
        {
          "distance": 32,
          "inside": {
            "kind": "kindof"
          }
        }
    "#;

    const INPUT_STRING_MESSAGE: &str = r#"
    syntax = "proto3";
    package Proto;

    message Inner { string kind = 1; }

    message Response {
      int32 distance = 1;
      Inner inside = 2;
    }
    "#;

    const MESSAGE_NAME: &str = "Proto.Response";

    const INPUT_STRING_HEX: &str = "082012080a066b696e646f66";
    const INPUT_STRING_BASE64: &str = "CCASCAoGa2luZG9m";

    fn get_input_as_bytes() -> Vec<u8> {
        hex::decode(INPUT_STRING_HEX).unwrap()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatProtobuf::new(
            get_input_as_bytes(), String::from(INPUT_STRING_MESSAGE), MESSAGE_NAME.to_string()).unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn to_vec_u8() {
        let input = PayloadFormatProtobuf::new(
            get_input_as_bytes(), String::from(INPUT_STRING_MESSAGE), MESSAGE_NAME.to_string()).unwrap();

        let result: Vec<u8> = Vec::from(input);

        assert_eq!(get_input_as_bytes(), result);
    }

    #[test]
    fn from_text_payload() {
        let input = PayloadFormatText::try_from(get_input_as_bytes()).unwrap();
        let result = PayloadFormatProtobuf::try_from(PayloadFormat::Text(input));

        assert!(result.is_err());
    }

    #[test]
    fn from_raw_payload() {
        let input = PayloadFormatRaw::try_from(get_input_as_bytes()).unwrap();
        let result = PayloadFormatProtobuf::try_from(PayloadFormat::Raw(input));

        assert!(result.is_err());
    }

    #[test]
    fn from_hex_payload() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatProtobuf::try_from(PayloadFormat::Hex(input));

        assert!(result.is_err());
    }

    #[test]
    fn from_base64_payload() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatProtobuf::try_from(PayloadFormat::Base64(input));

        assert!(result.is_err());
    }

    #[test]
    fn from_json_payload() {
        let input =
            PayloadFormatJson::try_from(Vec::<u8>::from(format!("{{\"content\": \"{}\"}}", INPUT_STRING_BASE64))).unwrap();
        let result = PayloadFormatProtobuf::try_from(PayloadFormat::Json(input));

        assert!(result.is_err());
    }

    #[test]
    fn from_yaml_payload() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(format!("content: \"{}\"", INPUT_STRING_BASE64))).unwrap();
        let result = PayloadFormatProtobuf::try_from(PayloadFormat::Yaml(input));

        assert!(result.is_err());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::from("not possible");
        let result =
            PayloadFormatProtobuf::convert_from(PayloadFormat::Text(input),
                                                String::from(INPUT_STRING_MESSAGE),
                                                MESSAGE_NAME);
        assert!(result.is_err());
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input_as_bytes()).unwrap();
        let result =
            PayloadFormatProtobuf::convert_from(PayloadFormat::Raw(input),
                                                String::from(INPUT_STRING_MESSAGE),
                                                MESSAGE_NAME).unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result =
            PayloadFormatProtobuf::convert_from(PayloadFormat::Hex(input),
                                                String::from(INPUT_STRING_MESSAGE),
                                                MESSAGE_NAME).unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result =
            PayloadFormatProtobuf::convert_from(PayloadFormat::Base64(input),
                                                String::from(INPUT_STRING_MESSAGE),
                                                MESSAGE_NAME).unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn from_yaml() {
        let input =
            PayloadFormatYaml::try_from(Vec::<u8>::from("content: input")).unwrap();
        let result =
            PayloadFormatProtobuf::convert_from(PayloadFormat::Yaml(input),
                                                String::from(INPUT_STRING_MESSAGE),
                                                MESSAGE_NAME);
        assert!(result.is_err());
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from("{\"content\":\"input\"}")).unwrap();
        let result =
            PayloadFormatProtobuf::convert_from(PayloadFormat::Json(input),
                                                String::from(INPUT_STRING_MESSAGE),
                                                MESSAGE_NAME);
        assert!(result.is_err());
    }

    fn extract_kind(result: &PayloadFormatProtobuf) -> String {
        let inner_message: &Value = &result.message_value.fields.iter().filter(|v| v.number == 2).collect::<Vec<&FieldValue>>().get(0).unwrap().value;

        let Value::Message(inner_message) = inner_message else {
            panic!("Inner message not found");
        };

        let kind: &Value = &inner_message.fields.iter().filter(|v| v.number == 1).collect::<Vec<&FieldValue>>().get(0).unwrap().value;

        let Value::String(kind) = kind else {
            panic!("Kind is not string");
        };

        kind.clone()
    }

    fn extract_distance(result: &PayloadFormatProtobuf) -> i32 {
        let distance: &Value = &result.message_value.fields.iter().filter(|v| v.number == 1).collect::<Vec<&FieldValue>>().get(0).unwrap().value;
        let Value::Int32(distance) = distance else {
            panic!("Distance is not i32");
        };

        *distance
    }
}

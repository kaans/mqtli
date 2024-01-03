use std::fmt::{Display, Formatter};
use std::string::FromUtf8Error;

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatText {
    content: String,
}

impl PayloadFormatText {
    fn decode_from_utf8(value: String) -> Vec<u8> {
        value.into_bytes()
    }

    fn encode_to_utf8(value: Vec<u8>) -> Result<String, FromUtf8Error> {
        String::from_utf8(value)
    }
}

/// Displays the UTF-8 encoded content.
impl Display for PayloadFormatText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// Encodes the given bytes as UTF-8 string.
impl TryFrom<Vec<u8>> for PayloadFormatText {
    type Error = PayloadFormatError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self {
            content: Self::encode_to_utf8(value)?,
        })
    }
}

/// Creates a new instance with the given UTF-8 encoded string as content.
/// The value is not modified, only moved to the new instance.
impl From<String> for PayloadFormatText {
    fn from(val: String) -> Self {
        Self { content: val }
    }
}

/// Creates a new instance with the given UTF-8 encoded string as content.
/// The value is not modified, only moved to the new instance.
impl From<&str> for PayloadFormatText {
    fn from(val: &str) -> Self {
        Self::from(val.to_string())
    }
}

/// Converts the utf-8 encoded content to its bytes.
///
/// # Examples
/// ```
/// let input = PayloadFormatText::from(String::from("INPUT"));
/// let v: Vec<u8> = Vec::from(input);
///
/// assert_eq!(vec![0x49, 0x4e, 0x50, 0x55, 0x54], v);
/// ```
impl From<PayloadFormatText> for Vec<u8> {
    fn from(val: PayloadFormatText) -> Self {
        PayloadFormatText::decode_from_utf8(<PayloadFormatText as Into<String>>::into(val))
    }
}

impl From<PayloadFormatText> for String {
    fn from(val: PayloadFormatText) -> Self {
        val.content
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatText {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => Ok(value),
            PayloadFormat::Raw(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Protobuf(value) => Ok(Self {
                content: protobuf::get_message_value(
                    value.context(),
                    value.message_value(),
                    0,
                    None,
                )?,
            }),
            PayloadFormat::Hex(value) => {
                let a: Vec<u8> = value.try_into()?;
                Self::try_from(a)
            }
            PayloadFormat::Base64(value) => {
                let a: Vec<u8> = value.try_into()?;
                Self::try_from(a)
            }
            PayloadFormat::Json(value) => {
                let Some(text_node) = value.content().get("content") else {
                    return Err(PayloadFormatError::CouldNotConvertFromJson(
                        "Attribute \"content\" not found".to_string(),
                    ));
                };

                let Some(text_node) = text_node.as_str() else {
                    return Err(PayloadFormatError::CouldNotConvertFromJson(
                        "Could not read attribute \"content\" as string".to_string(),
                    ));
                };

                Ok(Self::from(text_node))
            }
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

                Ok(Self::from(text_node))
            }
        }
    }
}

mod protobuf {
    use protofish::context::{Context, EnumField, MessageInfo};
    use protofish::decode::{FieldValue, MessageValue, Value};

    use crate::payload::PayloadFormatError;

    pub(super) fn get_message_value(
        context: &Context,
        message_value: &MessageValue,
        indent_level: u16,
        parent_field: Option<u64>,
    ) -> Result<String, PayloadFormatError> {
        let mut result = String::new();

        let message_info = context.resolve_message(message_value.msg_ref);

        let message_text = match parent_field {
            None => {
                format!("{} (Message)\n", message_info.full_name)
            }
            Some(parent_field) => {
                let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();
                format!(
                    "{indent_spaces}[{}] {} (Message)\n",
                    parent_field, message_info.full_name
                )
            }
        };
        result.push_str(&message_text);

        for field in &message_value.fields {
            let result_field = get_field_value(context, message_info, field, indent_level + 1)?;
            result.push_str(&result_field);
        }

        Ok(result)
    }

    fn get_field_value(
        context: &Context,
        message_response: &MessageInfo,
        field_value: &FieldValue,
        indent_level: u16,
    ) -> Result<String, PayloadFormatError> {
        let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();

        return match &message_response.get_field(field_value.number) {
            None => Err(PayloadFormatError::FieldNumberNotFoundInProtoFile(
                field_value.number,
            )),
            Some(field) => {
                let type_name = &field.name;

                let ret = match &field_value.value {
                    Value::Double(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Double)\n",
                            field.number, value
                        )
                    }
                    Value::Float(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Float)\n",
                            field.number, value
                        )
                    }
                    Value::Int32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Int32)\n",
                            field.number, value
                        )
                    }
                    Value::Int64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Int64)\n",
                            field.number, value
                        )
                    }
                    Value::UInt32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (UInt32)\n",
                            field.number, value
                        )
                    }
                    Value::UInt64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (UInt64)\n",
                            field.number, value
                        )
                    }
                    Value::SInt32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SInt32)\n",
                            field.number, value
                        )
                    }
                    Value::SInt64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SInt64)\n",
                            field.number, value
                        )
                    }
                    Value::Fixed32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Fixed32)\n",
                            field.number, value
                        )
                    }
                    Value::Fixed64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Fixed64)\n",
                            field.number, value
                        )
                    }
                    Value::SFixed32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SFixed32)\n",
                            field.number, value
                        )
                    }
                    Value::SFixed64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SFixed64)\n",
                            field.number, value
                        )
                    }
                    Value::Bool(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Bool)\n",
                            field.number, value
                        )
                    }
                    Value::String(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (String)\n",
                            field.number, value
                        )
                    }
                    Value::Bytes(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {:?} (Bytes)\n",
                            field.number, value
                        )
                    }
                    Value::Message(value) => {
                        get_message_value(context, value, indent_level, Some(field.number))?
                    }
                    Value::Packed(value) => {
                        format!(
                            "{indent_spaces}[{}] Unknown value encountered: {:?}\n",
                            field.number, value
                        )
                    }
                    Value::Enum(value) => {
                        let enum_value = context.resolve_enum(value.enum_ref);

                        format!(
                            "{indent_spaces}[{}] {type_name} = {:?} (Enum {})\n",
                            field.number,
                            enum_value
                                .get_field_by_value(value.value)
                                .unwrap_or(&EnumField::new("INVALID_VALUE".to_string(), 0))
                                .name,
                            enum_value.full_name
                        )
                    }
                    Value::Incomplete(number, value) => {
                        format!(
                            "{indent_spaces}[{}] Incomplete value encountered: {}: {:?}\n",
                            field.number, number, value
                        )
                    }
                    Value::Unknown(value) => {
                        format!(
                            "{indent_spaces}[{}] Unknown value encountered: {:?}\n",
                            field.number, value
                        )
                    }
                };

                Ok(ret)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::protobuf::PayloadFormatProtobuf;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::yaml::PayloadFormatYaml;

    use super::*;

    const INPUT_STRING: &str = "INPUT";
    const INPUT_STRING_HEX: &str = "494E505554";
    const INPUT_STRING_BASE64: &str = "SU5QVVQ=";
    const INPUT_STRING_PROTOBUF_AS_HEX: &str = "082012080a066b696e646f661801";
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
    }
    "#;
    const MESSAGE_NAME: &str = "Proto.Response";

    fn get_input() -> Vec<u8> {
        INPUT_STRING.into()
    }

    #[test]
    fn from_valid_vec_u8() {
        let result = PayloadFormatText::try_from(get_input()).unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_invalid_vec_u8() {
        let result = PayloadFormatText::try_from(vec![0xc3, 0x28]);

        assert!(result.is_err());
    }

    #[test]
    fn from_string() {
        let result = PayloadFormatText::from(INPUT_STRING.to_string());

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_string_ref() {
        let result = PayloadFormatText::from(INPUT_STRING);

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();

        let result: Vec<u8> = input.into();
        assert_eq!(INPUT_STRING.as_bytes(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::from(input);
        assert_eq!(INPUT_STRING.as_bytes(), result.as_slice());
    }

    #[test]
    fn to_string_into() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();

        let result: String = input.into();
        assert_eq!(INPUT_STRING, result.as_str());
    }

    #[test]
    fn to_string_from() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();

        let result: String = String::from(input);
        assert_eq!(INPUT_STRING, result.as_str());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Text(input)).unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(format!(
            "{{\"content\": \"{}\"}}",
            INPUT_STRING
        )))
        .unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_yaml() {
        let input =
            PayloadFormatYaml::try_from(Vec::<u8>::from(format!("content: \"{}\"", INPUT_STRING)))
                .unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_protobuf() {
        let input = PayloadFormatProtobuf::new(
            hex::decode(INPUT_STRING_PROTOBUF_AS_HEX).unwrap(),
            String::from(INPUT_STRING_MESSAGE),
            MESSAGE_NAME.to_string(),
        );
        eprintln!("input = {:?}", input);
        let result = PayloadFormatText::try_from(PayloadFormat::Protobuf(input.unwrap())).unwrap();

        assert_eq!("Proto.Response (Message)\n  [1] distance = 32 (Int32)\n  [2] Proto.Inner (Message)\n    [1] kind = kindof (String)\n  [3] position = \"POSITION_INSIDE\" (Enum Proto.Position)\n".to_owned(), result.content);
    }
}

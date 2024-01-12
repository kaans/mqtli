use protobuf::text_format::print_to_string_pretty;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::string::FromUtf8Error;

use crate::config::PayloadText;
use crate::payload::{convert_raw_type, PayloadFormat, PayloadFormatError};

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
        Self::try_from((value, &PayloadText::default()))
    }
}

impl TryFrom<(PayloadFormat, PayloadText)> for PayloadFormatText {
    type Error = PayloadFormatError;

    fn try_from((value, options): (PayloadFormat, PayloadText)) -> Result<Self, Self::Error> {
        Self::try_from((value, &options))
    }
}

impl TryFrom<(PayloadFormat, &PayloadText)> for PayloadFormatText {
    type Error = PayloadFormatError;

    fn try_from((value, options): (PayloadFormat, &PayloadText)) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => Ok(value),
            PayloadFormat::Raw(value) => Ok(Self {
                content: convert_raw_type(options.raw_as_type(), value.into()),
            }),
            PayloadFormat::Protobuf(value) => Ok(Self {
                content: print_to_string_pretty(value.content().deref()),
            }),
            PayloadFormat::Hex(value) => {
                let a: Vec<u8> = value.try_into()?;
                Ok(Self {
                    content: convert_raw_type(options.raw_as_type(), a),
                })
            }
            PayloadFormat::Base64(value) => {
                let a: Vec<u8> = value.try_into()?;
                Ok(Self {
                    content: convert_raw_type(options.raw_as_type(), a),
                })
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

#[cfg(test)]
mod tests {
    use crate::config::PayloadOptionRawFormat;
    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::protobuf::PayloadFormatProtobuf;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::yaml::PayloadFormatYaml;
    use lazy_static::lazy_static;
    use std::path::PathBuf;

    use super::*;

    const INPUT_STRING: &str = "INPUT";
    const INPUT_STRING_HEX: &str = "494e505554";
    const INPUT_STRING_BASE64: &str = "SU5QVVQ=";
    const INPUT_STRING_PROTOBUF_AS_HEX: &str = "082012080a066b696e646f6618012202c328";
    const MESSAGE_NAME: &str = "Response";

    lazy_static! {
        static ref INPUT_PATH_MESSAGE: PathBuf = PathBuf::from("test/data/message.proto");
    }

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
    fn from_raw_as_hex() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadText::default();
        let result = PayloadFormatText::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(INPUT_STRING_HEX.to_string(), result.content);
    }

    #[test]
    fn from_raw_as_base64() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadText::new(PayloadOptionRawFormat::Base64);
        let result = PayloadFormatText::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(INPUT_STRING_BASE64.to_string(), result.content);
    }

    #[test]
    fn from_raw_as_utf8() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadText::new(PayloadOptionRawFormat::Utf8);
        let result = PayloadFormatText::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(INPUT_STRING.to_string(), result.content);
    }

    #[test]
    fn from_hex_as_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX.to_owned(), result.content);
    }

    #[test]
    fn from_base64_as_hex() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX.to_owned(), result.content);
    }

    #[test]
    fn from_hex_as_base64() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatText::try_from((
            PayloadFormat::Hex(input),
            PayloadText::new(PayloadOptionRawFormat::Base64),
        ))
        .unwrap();

        assert_eq!(INPUT_STRING_BASE64.to_owned(), result.content);
    }

    #[test]
    fn from_base64_as_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatText::try_from((
            PayloadFormat::Base64(input),
            PayloadText::new(PayloadOptionRawFormat::Base64),
        ))
        .unwrap();

        assert_eq!(INPUT_STRING_BASE64.to_owned(), result.content);
    }

    #[test]
    fn from_hex_as_text() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatText::try_from((
            PayloadFormat::Hex(input),
            PayloadText::new(PayloadOptionRawFormat::Utf8),
        ))
        .unwrap();

        assert_eq!(INPUT_STRING.to_owned(), result.content);
    }

    #[test]
    fn from_base64_as_text() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatText::try_from((
            PayloadFormat::Base64(input),
            PayloadText::new(PayloadOptionRawFormat::Utf8),
        ))
        .unwrap();

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
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME.to_string(),
        );
        let result = PayloadFormatText::try_from(PayloadFormat::Protobuf(input.unwrap())).unwrap();

        assert_eq!("distance: 32\ninside {\n  kind: \"kindof\"\n}\nposition: POSITION_INSIDE\nraw: \"\\303(\"\n".to_owned(), result.content);
    }
}

use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use regex::Regex;

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatHex {
    content: String,
}

impl PayloadFormatHex {
    fn decode_from_hex<T: AsRef<[u8]>>(value: T) -> Result<Vec<u8>, PayloadFormatError> {
        Ok(hex::decode(value)?)
    }

    fn encode_to_hex(value: Vec<u8>) -> String {
        hex::encode(value)
    }
}

/// Displays the hex encoded content.
impl Display for PayloadFormatHex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// Converts the given `Vec<u8>` value to a hex encoded string.
impl From<Vec<u8>> for PayloadFormatHex {
    fn from(value: Vec<u8>) -> Self {
        Self { content: Self::encode_to_hex(value) }
    }
}

/// Creates a new instance with the given hex encoded string as content.
/// The value is not modified, only moved to the new instance. Thus, it
/// must already be encoded as hex, otherwise an error is returned.
impl TryFrom<String> for PayloadFormatHex {
    type Error = PayloadFormatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref REGEX_HEX: Regex = Regex::new("^([a-fA-F0-9]{2})*$").unwrap();
        }

        if !REGEX_HEX.is_match(value.as_str()) {
            return Err(PayloadFormatError::ValueIsNotValidHex(value));
        }

        Ok(Self {
            content: value,
        })
    }
}

/// Creates a new instance with the given hex encoded string as content.
/// The value is not modified, only moved to the new instance. Thus, it
/// must already be encoded as hex, otherwise an error is returned.
impl TryFrom<&str> for PayloadFormatHex {
    type Error = PayloadFormatError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

/// Converts the hex encoded content to its bytes.
///
/// # Examples
/// ```
/// let input = PayloadFormatHex::from(String::from("494e505554"));
/// let v: Vec<u8> = Vec::from(input);
///
/// assert_eq!(vec![0x49, 0x4e, 0x50, 0x55, 0x54], v);
/// ```
impl TryFrom<PayloadFormatHex> for Vec<u8> {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatHex) -> Result<Self, Self::Error> {
        PayloadFormatHex::decode_from_hex(value.content)
    }
}

/// Decodes into the string of the hex encoded value.
impl From<PayloadFormatHex> for String {
    fn from(val: PayloadFormatHex) -> Self {
        val.content
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatHex {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => {
                let a: Vec<u8> = value.into();
                Ok(Self::from(a))
            }
            PayloadFormat::Raw(value) => {
                let a: Vec<u8> = value.into();
                Ok(Self::from(a))
            }
            PayloadFormat::Protobuf(value) => {
                let a: Vec<u8> = value.into();
                Ok(Self::from(a))
            }
            PayloadFormat::Hex(value) => Ok(value),
            PayloadFormat::Base64(value) => {
                let a: Vec<u8> = value.try_into()?;
                Ok(Self::from(a))
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

                Self::try_from(text_node.to_owned())
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

                Self::try_from(text_node.to_owned())
            }
        }
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

    fn get_input() -> Vec<u8> {
        INPUT_STRING.into()
    }

    fn get_input_hex() -> String {
        INPUT_STRING_HEX.into()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatHex::from(get_input());

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn from_valid_string() {
        let result = PayloadFormatHex::try_from(get_input_hex()).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn from_invalid_string() {
        let result = PayloadFormatHex::try_from(get_input()).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatHex::from(get_input());

        let result: Vec<u8> = input.try_into().unwrap();
        assert_eq!(INPUT_STRING.as_bytes(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatHex::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::try_from(input).unwrap();
        assert_eq!(INPUT_STRING.as_bytes(), result.as_slice());
    }

    #[test]
    fn to_string_into() {
        let input = PayloadFormatHex::try_from(get_input_hex()).unwrap();

        let result: String = input.into();
        assert_eq!(INPUT_STRING_HEX, result.as_str());
    }

    #[test]
    fn to_string_from() {
        let input = PayloadFormatHex::try_from(get_input_hex()).unwrap();

        let result: String = String::from(input);
        assert_eq!(INPUT_STRING_HEX, result.as_str());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Text(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(get_input()).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(get_input()).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn from_json() {
        let input =
            PayloadFormatJson::try_from(Vec::<u8>::from(format!("{{\"content\": \"{}\"}}", INPUT_STRING_HEX))).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }

    #[test]
    fn from_yaml() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(format!("content: \"{}\"", INPUT_STRING_HEX))).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(INPUT_STRING_HEX, result.content);
    }
}

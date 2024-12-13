use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use regex::Regex;

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatHex {
    content: String,
}

impl PayloadFormatHex {
    pub fn decode_from_hex(self) -> Result<Vec<u8>, PayloadFormatError> {
        Ok(hex::decode(&self.content)?)
    }

    pub fn encode_to_hex(value: &Vec<u8>) -> String {
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
impl TryFrom<Vec<u8>> for PayloadFormatHex {
    type Error = PayloadFormatError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(String::from_utf8(value)?)
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

        Ok(Self { content: value })
    }
}

/// Converts the hex encoded content to its bytes.
///
/// # Examples
/// ```
/// use mqtlib::payload::hex::PayloadFormatHex;
/// let input = PayloadFormatHex::try_from(String::from("494e505554")).unwrap();
/// let v: Vec<u8> = Vec::from(input);
///
/// assert_eq!(vec![0x49, 0x4e, 0x50, 0x55, 0x54], v);
/// ```
impl From<PayloadFormatHex> for Vec<u8> {

    fn from(value: PayloadFormatHex) -> Self {
        value.content.into_bytes()
    }
}

/// Encodes into a string of the hex encoded value.
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
                Self::try_from(PayloadFormatHex::encode_to_hex(&Vec::<u8>::from(value)))
            }
            PayloadFormat::Raw(value) => {
                Self::try_from(PayloadFormatHex::encode_to_hex(&Vec::<u8>::from(value)))
            }
            PayloadFormat::Protobuf(value) => {
                Self::try_from(PayloadFormatHex::encode_to_hex(&Vec::<u8>::try_from(value)?))
            }
            PayloadFormat::Hex(value) => Ok(value),
            PayloadFormat::Base64(value) => {
                Self::try_from(PayloadFormatHex::encode_to_hex(&Vec::<u8>::from(value)))
            }
            PayloadFormat::Json(value) => {
                Self::try_from(PayloadFormatHex::encode_to_hex(&Vec::<u8>::from(value)))
            }
            PayloadFormat::Yaml(value) => {
                Self::try_from(PayloadFormatHex::encode_to_hex(&Vec::<u8>::try_from(value)?))
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

    fn get_input_hex_as_vec() -> Vec<u8> {
        get_input_hex().into_bytes()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatHex::try_from(get_input()).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn from_valid_string() {
        let result = PayloadFormatHex::try_from(get_input_hex()).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn from_invalid_string() {
        let result = PayloadFormatHex::try_from(get_input()).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatHex::try_from(get_input()).unwrap();

        let result: Vec<u8> = input.try_into().unwrap();
        assert_eq!(get_input_hex_as_vec(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatHex::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::try_from(input).unwrap();
        assert_eq!(get_input_hex_as_vec(), result.as_slice());
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

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(get_input()).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(get_input()).unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(format!(
            "{{\"content\": \"{}\"}}",
            INPUT_STRING_HEX
        )))
        .unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }

    #[test]
    fn from_yaml() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(format!(
            "content: \"{}\"",
            INPUT_STRING_HEX
        )))
        .unwrap();
        let result = PayloadFormatHex::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(get_input_hex(), result.content);
    }
}

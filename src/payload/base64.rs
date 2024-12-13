use std::fmt::{Display, Formatter};

use base64::engine::general_purpose;
use base64::Engine;

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatBase64 {
    content: String,
}

impl PayloadFormatBase64 {
    pub fn decode_from_base64(self: Self) -> Result<Vec<u8>, PayloadFormatError> {
        Ok(general_purpose::STANDARD.decode(self.content)?)
    }

    fn encode_to_base64(value: &Vec<u8>) -> String {
        general_purpose::STANDARD.encode(value)
    }

    fn is_valid_base64(value: &String) -> bool {
        general_purpose::STANDARD.decode(value).is_ok()
    }
}

/// Displays the base64 encoded content.
impl Display for PayloadFormatBase64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// Assumes the `Vec<u8>` value is a base64 encoded string.
impl TryFrom<Vec<u8>> for PayloadFormatBase64 {
    type Error = PayloadFormatError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(String::from_utf8(value)?)
    }
}

/// Creates a new instance with the given base64 encoded string as content.
/// The value is not modified, only moved to the new instance. Thus, it
/// must already be encoded as base64, otherwise an error is returned.
impl TryFrom<String> for PayloadFormatBase64 {
    type Error = PayloadFormatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid_base64(&value) {
            Ok(Self { content: value })
        } else {
            Err(PayloadFormatError::ValueIsNotValidBase64(value))
        }
    }
}

/// Decodes the base64 encoded value to its raw binary form.
///
/// # Examples
/// ```
/// use mqtlib::payload::base64::PayloadFormatBase64;
/// let input = PayloadFormatBase64::try_from(String::from("SU5QVVQ=")).unwrap();
/// let v: Vec<u8> = Vec::from(input);
///
/// assert_eq!(vec![0x49, 0x4e, 0x50, 0x55, 0x54], v);
/// ```
impl From<PayloadFormatBase64> for Vec<u8> {
    fn from(value: PayloadFormatBase64) -> Self {
        value.content.into_bytes()
    }
}

/// Encodes into a string of the base64 encoded value.
impl From<PayloadFormatBase64> for String {
    fn from(val: PayloadFormatBase64) -> Self {
        val.content
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatBase64 {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => Self::try_from(PayloadFormatBase64::encode_to_base64(
                &Vec::<u8>::from(value),
            )),
            PayloadFormat::Raw(value) => Self::try_from(PayloadFormatBase64::encode_to_base64(
                &Vec::<u8>::from(value),
            )),
            PayloadFormat::Protobuf(value) => Self::try_from(
                PayloadFormatBase64::encode_to_base64(&Vec::<u8>::try_from(value)?),
            ),
            PayloadFormat::Base64(value) => Ok(value),
            PayloadFormat::Hex(value) => Self::try_from(PayloadFormatBase64::encode_to_base64(
                &Vec::<u8>::from(value),
            )),
            PayloadFormat::Json(value) => Self::try_from(PayloadFormatBase64::encode_to_base64(
                &Vec::<u8>::from(value),
            )),
            PayloadFormat::Yaml(value) => Self::try_from(PayloadFormatBase64::encode_to_base64(
                &Vec::<u8>::try_from(value)?,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::text::PayloadFormatText;
    use crate::payload::yaml::PayloadFormatYaml;

    use super::*;

    const INPUT_STRING: &str = "INPUT";
    const INPUT_STRING_BASE64: &str = "SU5QVVQ="; // INPUT

    fn get_input() -> Vec<u8> {
        INPUT_STRING.into()
    }

    fn get_input_base64() -> String {
        INPUT_STRING_BASE64.into()
    }

    fn get_input_base64_as_vec() -> Vec<u8> {
        get_input_base64().into_bytes()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatBase64::try_from(get_input()).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }

    #[test]
    fn from_valid_string() {
        let result = PayloadFormatBase64::try_from(get_input_base64()).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }

    #[test]
    fn from_invalid_string() {
        let result = PayloadFormatBase64::try_from("INVALIDBASE64%&".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatBase64::try_from(get_input()).unwrap();

        let result: Vec<u8> = input.into();
        assert_eq!(get_input_base64_as_vec(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatBase64::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::try_from(input).unwrap();
        assert_eq!(get_input_base64_as_vec(), result.as_slice());
    }

    #[test]
    fn to_string_into() {
        let input = PayloadFormatBase64::try_from(get_input_base64()).unwrap();

        let result: String = input.into();
        assert_eq!(INPUT_STRING_BASE64, result.as_str());
    }

    #[test]
    fn to_string_from() {
        let input = PayloadFormatBase64::try_from(get_input_base64()).unwrap();

        let result: String = String::from(input);
        assert_eq!(INPUT_STRING_BASE64, result.as_str());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Text(input)).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(get_input()).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(get_input()).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(format!(
            "{{\"content\": \"{}\"}}",
            INPUT_STRING_BASE64
        )))
        .unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }

    #[test]
    fn from_yaml() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(format!(
            "content: \"{}\"",
            INPUT_STRING_BASE64
        )))
        .unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(get_input_base64(), result.content);
    }
}

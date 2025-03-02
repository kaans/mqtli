use crate::payload::{PayloadFormat, PayloadFormatError};
use derive_getters::Getters;
use std::fmt::{Display, Formatter};

/// Represents a lossy UTF-8 encoded String.
/// Any vector of u8 can be used to construct this String.
/// Non-UTF-8 characters will be ignored when rendering the
/// underlying vector as UTF-8.
#[derive(Clone, Debug, Getters)]
pub struct PayloadFormatText {
    pub content: Vec<u8>,
}

impl PayloadFormatText {
    fn decode_from_utf8(value: String) -> Vec<u8> {
        value.into_bytes()
    }

    fn encode_to_utf8(value: Vec<u8>) -> String {
        String::from_utf8_lossy(value.as_slice()).to_string()
    }
}

/// Displays the UTF-8 encoded content.
impl Display for PayloadFormatText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::encode_to_utf8(self.content.clone()))
    }
}

/// Encodes the given bytes as UTF-8 string.
impl From<Vec<u8>> for PayloadFormatText {
    fn from(value: Vec<u8>) -> Self {
        Self { content: value }
    }
}

/// Creates a new instance with the given UTF-8 encoded string as content.
/// The value is not modified, only moved to the new instance.
impl From<String> for PayloadFormatText {
    fn from(val: String) -> Self {
        Self {
            content: Self::decode_from_utf8(val),
        }
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
/// use mqtlib::payload::text::PayloadFormatText;
/// let input = PayloadFormatText::from(String::from("INPUT"));
/// let v: Vec<u8> = Vec::from(input);
///
/// assert_eq!(vec![0x49, 0x4e, 0x50, 0x55, 0x54], v);
/// ```
impl From<PayloadFormatText> for Vec<u8> {
    fn from(val: PayloadFormatText) -> Self {
        val.content
    }
}

impl From<PayloadFormatText> for String {
    fn from(val: PayloadFormatText) -> Self {
        PayloadFormatText::encode_to_utf8(val.content)
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatText {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => Ok(value),
            PayloadFormat::Raw(value) => Ok(Self {
                content: value.into(),
            }),
            PayloadFormat::Protobuf(value) => Ok(Self {
                content: value.to_string().into_bytes(),
            }),
            PayloadFormat::Hex(value) => Ok(Self {
                content: value.decode_from_hex()?,
            }),
            PayloadFormat::Base64(value) => Ok(Self {
                content: value.decode_from_base64()?,
            }),
            PayloadFormat::Json(value) => Ok(Self::from(value.to_string())),
            PayloadFormat::Yaml(value) => Ok(Self::from(value.to_string())),
            PayloadFormat::Sparkplug(value) => Ok(Self {
                content: value.to_string().into_bytes(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::protobuf::PayloadFormatProtobuf;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::yaml::PayloadFormatYaml;
    use lazy_static::lazy_static;
    use protobuf::text_format::print_to_string_pretty;
    use protobuf::MessageDyn;
    use std::path::PathBuf;

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

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_invalid_vec_u8() {
        let result = PayloadFormatText::try_from(vec![0xc3, 0x28]);

        assert!(result.is_ok());
    }

    #[test]
    fn from_string() {
        let result = PayloadFormatText::from(INPUT_STRING.to_string());

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_string_ref() {
        let result = PayloadFormatText::from(INPUT_STRING);

        assert_eq!(get_input(), result.content);
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

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_raw_as_hex() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_raw_as_base64() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_raw_as_utf8() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_hex_as_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_base64_as_hex() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_hex_as_base64() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_base64_as_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_hex_as_text() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_base64_as_text() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(format!(
            "{{\"content\": \"{}\"}}",
            INPUT_STRING
        )))
        .unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!("{\"content\":\"INPUT\"}".as_bytes(), result.content);
    }

    #[test]
    fn from_yaml() {
        let input =
            PayloadFormatYaml::try_from(Vec::<u8>::from(format!("content: \"{}\"", INPUT_STRING)))
                .unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!("content: INPUT\n".as_bytes(), result.content);
    }

    #[test]
    fn from_protobuf() {
        let input = PayloadFormatProtobuf::new(
            hex::decode(INPUT_STRING_PROTOBUF_AS_HEX).unwrap(),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME.to_string(),
        );
        let value = input.unwrap();
        let result = PayloadFormatText::try_from(PayloadFormat::Protobuf(value.clone())).unwrap();

        let msg: Box<dyn MessageDyn> = value.into();
        let pretty = print_to_string_pretty(&*msg);

        assert_eq!(pretty.as_bytes(), result.content);
    }
}

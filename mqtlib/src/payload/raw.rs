use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatRaw {
    content: Vec<u8>,
}

impl From<Vec<u8>> for PayloadFormatRaw {
    fn from(value: Vec<u8>) -> Self {
        Self { content: value }
    }
}

/// Returns the unaltered bytes of the content.
///
/// # Examples
/// ```
/// use mqtlib::payload::raw::PayloadFormatRaw;
/// let input = PayloadFormatRaw::from(vec![0x49, 0x4e, 0x50, 0x55, 0x54]);
/// let v: Vec<u8> = Vec::from(input);
///
/// assert_eq!(vec![0x49, 0x4e, 0x50, 0x55, 0x54], v);
/// ```
impl From<PayloadFormatRaw> for Vec<u8> {
    fn from(val: PayloadFormatRaw) -> Self {
        val.content
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatRaw {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => Ok(Self::from(Vec::<u8>::from(value))),
            PayloadFormat::Raw(value) => Ok(value),
            PayloadFormat::Protobuf(value) => Ok(Self::from(Vec::<u8>::try_from(value)?)),
            PayloadFormat::Hex(value) => Ok(Self::from(value.decode_from_hex()?)),
            PayloadFormat::Base64(value) => Ok(Self::from(value.decode_from_base64()?)),
            PayloadFormat::Json(value) => Ok(Self::from(Vec::<u8>::from(value))),
            PayloadFormat::Yaml(value) => Ok(Self::from(Vec::<u8>::try_from(value)?)),
            PayloadFormat::Sparkplug(value) => Ok(Self::from(Vec::<u8>::try_from(value)?)),
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
    const INPUT_STRING_HEX: &str = "494E505554";
    // INPUT
    const INPUT_STRING_BASE64: &str = "SU5QVVQ="; // INPUT

    fn get_input() -> Vec<u8> {
        INPUT_STRING.into()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatRaw::try_from(get_input()).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();

        let result: Vec<u8> = input.into();
        assert_eq!(get_input(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::from(input);
        assert_eq!(get_input(), result.as_slice());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::try_from(get_input()).unwrap();
        let result = PayloadFormatRaw::try_from(PayloadFormat::Text(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();
        let result = PayloadFormatRaw::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatRaw::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatRaw::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_input(), result.content);
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(format!(
            "{{\"content\": \"{}\"}}",
            INPUT_STRING
        )))
        .unwrap();
        let result = PayloadFormatRaw::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!("{\"content\":\"INPUT\"}".as_bytes(), result.content);
    }

    #[test]
    fn from_yaml() {
        let input =
            PayloadFormatYaml::try_from(Vec::<u8>::from(format!("content: {}", INPUT_STRING)))
                .unwrap();
        let result = PayloadFormatRaw::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!("content: INPUT\n".as_bytes(), result.content);
    }
}

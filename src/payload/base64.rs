use base64::engine::general_purpose;
use base64::Engine;

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatBase64 {
    content: Vec<u8>,
}

impl From<Vec<u8>> for PayloadFormatBase64 {
    fn from(value: Vec<u8>) -> Self {
        Self { content: value }
    }
}

impl TryFrom<String> for PayloadFormatBase64 {
    type Error = PayloadFormatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            content: general_purpose::STANDARD.decode(Vec::<u8>::from(value))?,
        })
    }
}

impl From<PayloadFormatBase64> for Vec<u8> {
    fn from(val: PayloadFormatBase64) -> Self {
        val.content
    }
}

impl From<PayloadFormatBase64> for String {
    fn from(val: PayloadFormatBase64) -> Self {
        general_purpose::STANDARD.encode(val.content)
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatBase64 {
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
            PayloadFormat::Base64(value) => Ok(value),
            PayloadFormat::Hex(value) => {
                let a: Vec<u8> = value.into();
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

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatBase64::from(get_input());

        assert_eq!(Vec::from(INPUT_STRING), result.content);
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatBase64::try_from(get_input()).unwrap();

        let result: Vec<u8> = input.into();
        assert_eq!(INPUT_STRING.as_bytes(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatBase64::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::from(input);
        assert_eq!(INPUT_STRING.as_bytes(), result.as_slice());
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

        assert_eq!(Vec::from(INPUT_STRING), result.content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input()).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Raw(input)).unwrap();

        assert_eq!(Vec::from(INPUT_STRING), result.content);
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from("494E505554".to_owned()).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(Vec::from(INPUT_STRING), result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from("SU5QVVQ=".to_owned()).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(Vec::from(INPUT_STRING), result.content);
    }

    #[test]
    fn from_json() {
        let input =
            PayloadFormatJson::try_from(Vec::<u8>::from("{\"content\": \"SU5QVVQ=\"}")).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(Vec::from(INPUT_STRING), result.content);
    }

    #[test]
    fn from_yaml() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from("content: \"SU5QVVQ=\"")).unwrap();
        let result = PayloadFormatBase64::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(Vec::from(INPUT_STRING), result.content);
    }
}

use std::fmt::{Display, Formatter};

use derive_getters::Getters;
use log::error;
use serde_yaml::{from_slice, from_str, Value};

use crate::config::PayloadYaml;
use crate::payload::json::PayloadFormatJson;
use crate::payload::{convert_raw_type, PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug, Getters)]
pub struct PayloadFormatYaml {
    content: Value,
}

impl PayloadFormatYaml {
    fn decode_from_yaml_payload(value: &PayloadFormatYaml) -> serde_yaml::Result<String> {
        serde_yaml::to_string(&value.content)
    }

    fn encode_to_yaml(value: Vec<u8>) -> serde_yaml::Result<Value> {
        from_slice(value.as_slice())
    }
}

/// Displays the hex encoded content.
impl Display for PayloadFormatYaml {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Self::decode_from_yaml_payload(self)
                .map_err(|e| {
                    error!("Cannot display yaml, error while decoding: {:?}", e);
                    ""
                })
                .unwrap()
        )
    }
}

impl TryFrom<Vec<u8>> for PayloadFormatYaml {
    type Error = PayloadFormatError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self {
            content: Self::encode_to_yaml(value)?,
        })
    }
}

impl TryFrom<String> for PayloadFormatYaml {
    type Error = PayloadFormatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.into_bytes())
    }
}

impl From<Value> for PayloadFormatYaml {
    fn from(val: Value) -> Self {
        Self { content: val }
    }
}

impl TryFrom<PayloadFormatYaml> for Vec<u8> {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatYaml) -> Result<Self, Self::Error> {
        Ok(<PayloadFormatYaml as TryInto<String>>::try_into(value)?.into_bytes())
    }
}

impl TryFrom<PayloadFormatYaml> for String {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatYaml) -> Result<Self, Self::Error> {
        Ok(PayloadFormatYaml::decode_from_yaml_payload(&value)?)
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatYaml {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        Self::try_from((value, &PayloadYaml::default()))
    }
}

impl TryFrom<(PayloadFormat, &PayloadYaml)> for PayloadFormatYaml {
    type Error = PayloadFormatError;

    fn try_from((value, options): (PayloadFormat, &PayloadYaml)) -> Result<Self, Self::Error> {
        fn convert_from_value(value: String) -> Result<PayloadFormatYaml, PayloadFormatError> {
            let yaml: Value = from_str(format!("content: {}", value).as_str())?;
            Ok(PayloadFormatYaml::from(yaml))
        }

        match value {
            PayloadFormat::Text(value) => convert_from_value(value.into()),
            PayloadFormat::Raw(value) => {
                convert_from_value(convert_raw_type(options.raw_as_type(), value.into()))
            }
            PayloadFormat::Protobuf(value) => {
                let json = PayloadFormatJson::try_from(PayloadFormat::Protobuf(value))?;
                Ok(Self {
                    content: serde_json::from_value::<Value>(json.content().clone())?,
                })
            }
            PayloadFormat::Hex(value) => convert_from_value(value.into()),
            PayloadFormat::Base64(value) => convert_from_value(value.into()),
            PayloadFormat::Yaml(value) => Ok(value),
            PayloadFormat::Json(value) => Ok(Self {
                content: serde_json::from_value::<Value>(value.content().clone())?,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::PayloadOptionRawFormat;
    use lazy_static::lazy_static;
    use serde_yaml::from_str;
    use std::path::PathBuf;

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
    const MESSAGE_NAME: &str = "Response";

    lazy_static! {
        static ref INPUT_PATH_MESSAGE: PathBuf = PathBuf::from("test/data/message.proto");
    }

    fn get_input() -> Vec<u8> {
        get_input_yaml(INPUT_STRING).into()
    }

    fn get_input_yaml(value: &str) -> String {
        format!("content: {}\n", value)
    }

    fn get_yaml_value(value: &str) -> Value {
        from_str(get_input_yaml(value).as_str()).unwrap()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatYaml::try_from(get_input()).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING), result.content);
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatYaml::try_from(get_input()).unwrap();

        let result: Vec<u8> = input.try_into().unwrap();
        assert_eq!(get_input_yaml(INPUT_STRING).as_bytes(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatYaml::try_from(get_input()).unwrap();

        let result: Vec<u8> = Vec::try_from(input).unwrap();
        assert_eq!(get_input_yaml(INPUT_STRING).as_bytes(), result.as_slice());
    }

    #[test]
    fn to_string_into() {
        let input = PayloadFormatYaml::try_from(get_input()).unwrap();

        let result: String = input.try_into().unwrap();
        assert_eq!(get_input_yaml(INPUT_STRING), result.as_str());
    }

    #[test]
    fn to_string_from() {
        let input = PayloadFormatYaml::try_from(get_input()).unwrap();

        let result: String = String::try_from(input).unwrap();
        assert_eq!(get_input_yaml(INPUT_STRING), result.as_str());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::from(INPUT_STRING.to_string());
        let result = PayloadFormatYaml::try_from(PayloadFormat::Text(input)).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING), result.content);
    }

    #[test]
    fn from_raw_as_hex() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadYaml::default();
        let result = PayloadFormatYaml::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING_HEX), result.content);
    }

    #[test]
    fn from_raw_as_base64() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadYaml::new(PayloadOptionRawFormat::Base64);
        let result = PayloadFormatYaml::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING_BASE64), result.content);
    }

    #[test]
    fn from_raw_as_utf8() {
        let input = PayloadFormatRaw::try_from(Vec::from(INPUT_STRING)).unwrap();
        let options = PayloadYaml::new(PayloadOptionRawFormat::Utf8);
        let result = PayloadFormatYaml::try_from((PayloadFormat::Raw(input), &options)).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING), result.content);
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatYaml::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING_HEX), result.content);
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatYaml::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING_BASE64), result.content);
    }

    #[test]
    fn from_yaml() {
        let input =
            PayloadFormatYaml::try_from(Vec::<u8>::from(get_input_yaml(INPUT_STRING))).unwrap();
        let result = PayloadFormatYaml::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!(get_yaml_value(INPUT_STRING), result.content);
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(format!(
            "{{\"content\":\"{}\"}}",
            INPUT_STRING
        )))
        .unwrap();
        let result = PayloadFormatYaml::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(
            INPUT_STRING,
            result.content.get("content").unwrap().as_str().unwrap()
        );
    }

    #[test]
    fn from_json_complex() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(format!(
            "{{\"size\": 54.3, \"name\":\"{}\"}}",
            INPUT_STRING
        )))
        .unwrap();
        let result = PayloadFormatYaml::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!(54.3, result.content.get("size").unwrap().as_f64().unwrap());
        assert_eq!(
            INPUT_STRING,
            result.content.get("name").unwrap().as_str().unwrap()
        );
    }

    #[test]
    fn from_json_array() {
        let input_string = r#"
            {
                "size": 54.3,
                "name": "full name",
                "colors": [
                    "red",
                    "blue",
                    "green"
                ]
            }
        "#;

        let input = PayloadFormatJson::try_from(Vec::<u8>::from(input_string)).unwrap();
        let result = PayloadFormatYaml::try_from(PayloadFormat::Json(input)).unwrap();

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
                .as_sequence()
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
                .as_sequence()
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
                .as_sequence()
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
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME.to_string(),
        );
        let result = PayloadFormatYaml::try_from(PayloadFormat::Protobuf(input.unwrap())).unwrap();

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
                .as_mapping()
                .unwrap()
                .get("kind")
                .unwrap()
                .as_str()
                .unwrap()
        );
    }
}

use crate::payload::json::PayloadFormatJson;
use crate::payload::{PayloadFormat, PayloadFormatError};
use derive_getters::Getters;
use protobuf::text_format::print_to_string_pretty;
use protobuf::Message;
use protobuf_json_mapping::parse_from_str;
use std::fmt::{Display, Formatter};

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}

use crate::payload::sparkplug::protos::sparkplug_b::Payload as SparkplugPayload;

#[derive(Clone, Debug, Getters)]
pub struct PayloadFormatSparkplug {
    pub content: SparkplugPayload,
}

impl Display for PayloadFormatSparkplug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", print_to_string_pretty(&self.content))
    }
}

impl From<SparkplugPayload> for PayloadFormatSparkplug {
    fn from(value: SparkplugPayload) -> Self {
        Self { content: value }
    }
}

/// Returns the unaltered bytes of the content.
///
/// # Examples
/// ```
/// use mqtlib::payload::sparkplug::PayloadFormatSparkplug;
/// let input = PayloadFormatSparkplug::try_from(vec![0x08, 0xfa, 0x8a, 0xf3, 0xa2, 0x02, 0x12, 0x17, 0x0a, 0x08, 0x68, 0x75, 0x6d, 0x69, 0x64, 0x69, 0x74, 0x79, 0x18, 0xfb, 0x8a, 0xf3, 0xa2, 0x02, 0x20, 0x09, 0x65, 0xcd, 0xcc, 0x8f, 0x42, 0x18, 0x8c, 0x01]).unwrap();
/// let v: Vec<u8> = Vec::try_from(input).unwrap();
///
/// assert_eq!(vec![0x08, 0xfa, 0x8a, 0xf3, 0xa2, 0x02, 0x12, 0x17, 0x0a, 0x08, 0x68, 0x75, 0x6d, 0x69, 0x64, 0x69, 0x74, 0x79, 0x18, 0xfb, 0x8a, 0xf3, 0xa2, 0x02, 0x20, 0x09, 0x65, 0xcd, 0xcc, 0x8f, 0x42, 0x18, 0x8c, 0x01], v);
/// ```
impl TryFrom<PayloadFormatSparkplug> for Vec<u8> {
    type Error = PayloadFormatError;
    fn try_from(val: PayloadFormatSparkplug) -> Result<Self, Self::Error> {
        val.content
            .write_to_bytes()
            .map_err(PayloadFormatError::from)
    }
}

impl TryFrom<Vec<u8>> for PayloadFormatSparkplug {
    type Error = PayloadFormatError;
    fn try_from(val: Vec<u8>) -> Result<Self, Self::Error> {
        let msg: SparkplugPayload = Message::parse_from_bytes(val.as_slice())?;
        Ok(PayloadFormatSparkplug::from(msg))
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatSparkplug {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => Ok(Self::try_from(Vec::<u8>::from(value))?),
            PayloadFormat::Raw(value) => Ok(Self::try_from(Vec::<u8>::from(value))?),
            PayloadFormat::Protobuf(value) => Ok(Self::try_from(Vec::<u8>::try_from(value)?)?),
            PayloadFormat::Hex(value) => Ok(Self::try_from(value.decode_from_hex()?)?),
            PayloadFormat::Base64(value) => Ok(Self::try_from(value.decode_from_base64()?)?),
            PayloadFormat::Json(value) => {
                let payload: SparkplugPayload = parse_from_str(value.to_string().as_str())?;

                Ok(Self::from(payload))
            }
            PayloadFormat::Yaml(value) => {
                let json = PayloadFormatJson::try_from(PayloadFormat::Yaml(value))?;
                let payload: SparkplugPayload = parse_from_str(json.to_string().as_str())?;
                Ok(Self::from(payload))
            }
            PayloadFormat::Sparkplug(value) => Ok(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::sparkplug::PayloadFormatSparkplug;
    use crate::payload::text::PayloadFormatText;
    use crate::payload::yaml::PayloadFormatYaml;
    use crate::payload::PayloadFormat;

    const INPUT_STRING_JSON: &str = "\
    {
      \"metrics\":[
        {
          \"datatype\":9,
          \"floatValue\":71.9,
          \"name\":\"humidity\",
          \"timestamp\":\"610059643\"
        }
      ],
      \"seq\":\"140\",
      \"timestamp\":\"610059642\"
    }";
    const INPUT_STRING_YAML: &str = "\
    ---
    metrics:
    - datatype: 9
      floatValue: 71.9
      name: humidity
      timestamp: '610059643'
    seq: '140'
    timestamp: '610059642'
    ";
    const INPUT_STRING_HEX: &str =
        "08fa8af3a20212170a0868756d696469747918fb8af3a202200965cdcc8f42188c01";
    const INPUT_STRING_BASE64: &str = "CPqK86ICEhcKCGh1bWlkaXR5GPuK86ICIAllzcyPQhiMAQ==";

    fn get_input_as_bytes() -> Vec<u8> {
        hex::decode(INPUT_STRING_HEX).unwrap()
    }
    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatSparkplug::try_from(get_input_as_bytes()).unwrap();

        assert_eq!("humidity", result.content.metrics[0].clone().name.unwrap());
    }

    #[test]
    fn to_vec_u8_into() {
        let input = PayloadFormatSparkplug::try_from(get_input_as_bytes()).unwrap();

        let result: Vec<u8> = input.try_into().unwrap();
        assert_eq!(get_input_as_bytes(), result.as_slice());
    }

    #[test]
    fn to_vec_u8_from() {
        let input = PayloadFormatSparkplug::try_from(get_input_as_bytes()).unwrap();

        let result: Vec<u8> = Vec::try_from(input).unwrap();
        assert_eq!(get_input_as_bytes(), result.as_slice());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::from("not possible");
        let result = PayloadFormatSparkplug::try_from(PayloadFormat::Text(input));
        assert!(result.is_err());
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatSparkplug::try_from(get_input_as_bytes()).unwrap();
        let result = PayloadFormatSparkplug::try_from(PayloadFormat::Sparkplug(input)).unwrap();

        assert_eq!("humidity", result.content.metrics[0].clone().name.unwrap());
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatSparkplug::try_from(PayloadFormat::Hex(input)).unwrap();

        assert_eq!("humidity", result.content.metrics[0].clone().name.unwrap());
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatSparkplug::try_from(PayloadFormat::Base64(input)).unwrap();

        assert_eq!("humidity", result.content.metrics[0].clone().name.unwrap());
    }

    #[test]
    fn from_json() {
        let input = PayloadFormatJson::try_from(Vec::<u8>::from(INPUT_STRING_JSON)).unwrap();
        let result = PayloadFormatSparkplug::try_from(PayloadFormat::Json(input)).unwrap();

        assert_eq!("humidity", result.content.metrics[0].clone().name.unwrap());
    }

    #[test]
    fn from_yaml() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from(INPUT_STRING_YAML)).unwrap();
        let result = PayloadFormatSparkplug::try_from(PayloadFormat::Yaml(input)).unwrap();

        assert_eq!("humidity", result.content.metrics[0].clone().name.unwrap());
    }
}

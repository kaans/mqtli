use std::str::from_utf8;

use base64::Engine;
use base64::engine::general_purpose;
use bytes::Bytes;
use rumqttc::v5::mqttbytes::v5::Publish;
use serde_json::json;
use crate::config::OutputFormat;

use crate::payload::PayloadError;

pub struct PayloadTextHandler {}

impl PayloadTextHandler {
    pub fn handle_publish(value: &Publish, output_format: &OutputFormat) -> Result<Vec<u8>, PayloadError> {
        Self::encode_to_output_format(&value.payload, output_format)
    }

    fn encode_to_output_format(content: &Bytes, output_format: &OutputFormat) -> Result<Vec<u8>, PayloadError> {
        match output_format {
            OutputFormat::Text => {
                Self::convert_to_text(content)
            }
            OutputFormat::Json => {
                Self::convert_to_json(content)
            }
            OutputFormat::Yaml => {
                Self::convert_to_yaml(content)
            }
            OutputFormat::Hex => {
                Self::convert_to_hex(content)
            }
            OutputFormat::Base64 => {
                Self::convert_to_base64(content)
            }
            OutputFormat::Raw => {
                Ok(content.to_vec())
            }
        }
    }

    fn convert_to_text(content: &Bytes) -> Result<Vec<u8>, PayloadError> {
        match from_utf8(content) {
            Ok(content) => Ok(content.to_string().into_bytes()),
            Err(e) => {
                Err(PayloadError::CouldNotConvertToUtf8(e))
            }
        }
    }

    fn convert_to_json(content: &Bytes) -> Result<Vec<u8>, PayloadError> {
        match from_utf8(content) {
            Ok(content) => {
                let json = json!({
                     "text": content
                 });

                Ok(json.to_string().into_bytes())
            }
            Err(e) => {
                Err(PayloadError::CouldNotConvertToUtf8(e))
            }
        }
    }

    fn convert_to_yaml(content: &Bytes) -> Result<Vec<u8>, PayloadError> {
        match from_utf8(content) {
            Ok(content) => {
                let mut mapping = serde_yaml::Mapping::with_capacity(1);
                mapping.insert(serde_yaml::Value::String("text".to_string()), serde_yaml::Value::String(content.to_string()));
                let yaml = serde_yaml::Value::Mapping(mapping);

                match serde_yaml::to_string(&yaml) {
                    Ok(yaml) => Ok(yaml.into_bytes()),
                    Err(e) => {
                        Err(PayloadError::CouldNotConvertToYaml(e))
                    }
                }
            }
            Err(e) => {
                Err(PayloadError::CouldNotConvertToUtf8(e))
            }
        }
    }

    fn convert_to_hex(content: &Bytes) -> Result<Vec<u8>, PayloadError> {
        let hex = hex::encode_upper(content.to_vec());
        Ok(hex.into_bytes())
    }

    fn convert_to_base64(content: &Bytes) -> Result<Vec<u8>, PayloadError> {
        let base64 = general_purpose::STANDARD_NO_PAD.encode(content);
        Ok(base64.into_bytes())
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use crate::payload::text::PayloadTextHandler;

    #[test]
    fn convert_hex() {
        let input = Bytes::from("Input text");
        let output = PayloadTextHandler::convert_to_hex(&input);
        assert_eq!("496E7075742074657874".as_bytes(), output.unwrap());
    }

    #[test]
    fn convert_base64() {
        let input = Bytes::from("Input text");
        let output = PayloadTextHandler::convert_to_base64(&input);
        assert_eq!("SW5wdXQgdGV4dA".as_bytes(), output.unwrap());
    }

    #[test]
    fn convert_yaml() {
        let input = Bytes::from("Input text");
        let output = PayloadTextHandler::convert_to_yaml(&input);
        assert_eq!("text: Input text\n".as_bytes(), output.unwrap());
    }

    #[test]
    fn convert_json() {
        let input = Bytes::from("Input text");
        let output = PayloadTextHandler::convert_to_json(&input);
        assert_eq!("{\"text\":\"Input text\"}".as_bytes(), output.unwrap());
    }
}
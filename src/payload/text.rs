use std::str::from_utf8;

use rumqttc::v5::mqttbytes::v5::Publish;
use serde_json::json;
use crate::config::mqtli_config::OutputFormat;

use crate::payload::{PayloadError};

pub struct PayloadTextHandler {}

impl PayloadTextHandler {
    pub fn handle_publish(value: &Publish, output_format: &OutputFormat) -> Result<Vec<u8>, PayloadError> {
        match from_utf8(value.payload.as_ref()) {
            Ok(content) => {
                Self::encode_to_output_format(content, output_format)
            }
            Err(e) => {
                Err(PayloadError::CouldNotConvertToUtf8(e))
            }
        }
    }

    fn encode_to_output_format(content: &str, output_format: &OutputFormat) -> Result<Vec<u8>, PayloadError> {
        match output_format {
            OutputFormat::Plain => {
                Ok(content.to_string().into_bytes())
            }
            OutputFormat::Json => {
                Self::convert_to_json(content)
            }
        }
    }

    fn convert_to_json(content: &str) -> Result<Vec<u8>, PayloadError> {
        let json = json!({
         "text": content
     });

        Ok(json.to_string().into_bytes())
    }
}

use std::str::from_utf8;

use rumqttc::v5::mqttbytes::v5::Publish;

use crate::payload::{OutputFormat, PayloadError};

pub struct PayloadTextHandler {}

impl PayloadTextHandler {
    pub fn handle_publish(value: &Publish, output_format: OutputFormat) -> Result<Vec<u8>, PayloadError> {
        match from_utf8(value.payload.as_ref()) {
            Ok(content) => {
                Self::encode_to_output_format(content, output_format)
            }
            Err(e) => {
                Err(PayloadError::CouldNotConvertToUtf8(e))
            }
        }
    }

    fn encode_to_output_format(content: &str, output_format: OutputFormat) -> Result<Vec<u8>, PayloadError> {
        match output_format {
            OutputFormat::PLAIN => {
                Ok(content.to_string().into_bytes())
            }
            _ => {
                Err(PayloadError::OutputFormatNotSupported(output_format))
            }
        }
    }
}
use crate::config::mqtli_config::{PayloadText, PayloadType, PublishInputTypeContentPath};
use crate::input::{InputError, read_from_path};

pub struct InputConverterText {}

impl InputConverterText {
    pub fn convert(input: &PublishInputTypeContentPath, output_format: &PayloadType) -> Result<Vec<u8>, InputError> {
        let content = if let Some(content) = input.content() {
            Vec::from(content.as_str())
        } else {
            if let Some(path) = input.path() {
                read_from_path(path)?
            } else {
                return Err(InputError::EitherContentOrPathMustBeGiven);
            }
        };

        match output_format {
            PayloadType::Text(options) => Self::convert_to_text(content, options),
            PayloadType::Protobuf(_options)
            => Err(InputError::ConversionNotPossible(String::from("text"), String::from("protobuf")))
        }
    }

    pub fn convert_to_text(input: Vec<u8>, _options: &PayloadText) -> Result<Vec<u8>, InputError> {
        match String::from_utf8(input) {
            Ok(content) => Ok(content.into_bytes()),
            Err(e) => Err(InputError::CouldNotDecodeUtf8(e))
        }
    }
}
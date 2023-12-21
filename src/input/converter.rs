use crate::config::mqtli_config::{PayloadType, PublishInputType};
use crate::input::InputError;
use crate::input::raw::InputConverterRaw;
use crate::input::text::InputConverterText;

pub struct InputConverter {}

impl InputConverter {
    pub fn convert_input(input: &PublishInputType, output_format: &PayloadType) -> Result<Vec<u8>, InputError> {
        match input {
            PublishInputType::Text(input) => {
                InputConverterText::convert(input, output_format)
            }
            PublishInputType::Raw(input) => {
                InputConverterRaw::convert(input, output_format)
            }
        }
    }
}
use std::path::PathBuf;

use derive_getters::Getters;
use protobuf::reflect::{FileDescriptor, MessageDescriptor};
use protobuf::MessageDyn;
use protobuf_json_mapping::parse_dyn_from_str;

use crate::config::PayloadProtobuf;
use crate::payload::json::PayloadFormatJson;
use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Debug, Getters)]
pub struct PayloadFormatProtobuf {
    content: Box<dyn MessageDyn>,
}

impl PayloadFormatProtobuf {
    pub fn new(
        content: Vec<u8>,
        definition_file: &PathBuf,
        message_name: String,
    ) -> Result<Self, PayloadFormatError> {
        let result = Self::convert_from_vec(content, definition_file, message_name.as_str())?;

        Ok(Self { content: result })
    }

    fn convert_from_vec(
        content: Vec<u8>,
        definition_file: &PathBuf,
        message_name: &str,
    ) -> Result<Box<dyn MessageDyn>, PayloadFormatError> {
        let md = Self::get_message_descriptor(definition_file, message_name)?;

        let result = md.parse_from_bytes(content.as_slice())?;
        Ok(result)
    }

    pub fn convert_from(
        payload: PayloadFormat,
        definition_file: &PathBuf,
        message_name: &str,
    ) -> Result<Self, PayloadFormatError> {
        let content: Box<dyn MessageDyn> = match payload {
            PayloadFormat::Text(_value) => {
                return Err(PayloadFormatError::ConversionNotPossible(
                    "text".to_string(),
                    "protobuf".to_string(),
                ));
            }
            PayloadFormat::Raw(value) => {
                Self::convert_from_vec(Vec::from(value), definition_file, message_name)?
            }
            PayloadFormat::Protobuf(value) => value.content,
            PayloadFormat::Hex(value) => {
                Self::convert_from_vec(Vec::from(value), definition_file, message_name)?
            }
            PayloadFormat::Base64(value) => {
                Self::convert_from_vec(Vec::from(value), definition_file, message_name)?
            }
            PayloadFormat::Json(value) => {
                Self::convert_from_json(value, definition_file, message_name)?
            }
            PayloadFormat::Yaml(value) => {
                let json = PayloadFormatJson::try_from(PayloadFormat::Yaml(value))?;
                Self::convert_from_json(json, definition_file, message_name)?
            }
        };

        Ok(Self { content })
    }

    fn get_message_descriptor(
        proto_message_path: &PathBuf,
        message_name: &str,
    ) -> Result<MessageDescriptor, PayloadFormatError> {
        let include_path = proto_message_path
            .parent()
            .ok_or(PayloadFormatError::CouldNotOpenProtobufDefinitionFile)?;
        let proto_file = protobuf_parse::Parser::new()
            .pure()
            .include(include_path)
            .input(proto_message_path)
            .parse_and_typecheck()
            .unwrap()
            .file_descriptors
            .pop()
            .unwrap();

        let dynamic_file_descriptor = FileDescriptor::new_dynamic(proto_file.clone(), &[])?;
        dynamic_file_descriptor
            .message_by_package_relative_name(message_name)
            .ok_or(PayloadFormatError::ProtobufMessageNotFound(
                message_name.to_string(),
            ))
    }

    fn convert_from_json(
        value: PayloadFormatJson,
        definition_file: &PathBuf,
        message_name: &str,
    ) -> Result<Box<dyn MessageDyn>, PayloadFormatError> {
        let md = Self::get_message_descriptor(definition_file, message_name)?;

        Ok(parse_dyn_from_str(&md, value.to_string().as_str())?)
    }
}

/// Encodes the protobuf message and returns its byte representation.
impl TryFrom<PayloadFormatProtobuf> for Vec<u8> {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatProtobuf) -> Result<Self, Self::Error> {
        Ok(value.content.write_to_bytes_dyn()?)
    }
}

impl TryFrom<(PayloadFormat, &PayloadProtobuf)> for PayloadFormatProtobuf {
    type Error = PayloadFormatError;

    fn try_from((value, options): (PayloadFormat, &PayloadProtobuf)) -> Result<Self, Self::Error> {
        Self::convert_from(value, options.definition(), options.message())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use lazy_static::lazy_static;

    use crate::payload::base64::PayloadFormatBase64;
    use crate::payload::hex::PayloadFormatHex;
    use crate::payload::json::PayloadFormatJson;
    use crate::payload::raw::PayloadFormatRaw;
    use crate::payload::text::PayloadFormatText;
    use crate::payload::yaml::PayloadFormatYaml;

    use super::*;

    const MESSAGE_NAME: &str = "Response";

    const INPUT_STRING_HEX: &str = "082012080a066b696e646f66";
    const INPUT_STRING_BASE64: &str = "CCASCAoGa2luZG9m";

    lazy_static! {
        static ref INPUT_PATH_MESSAGE: PathBuf = PathBuf::from("test/data/message.proto");
    }

    fn get_input_as_bytes() -> Vec<u8> {
        hex::decode(INPUT_STRING_HEX).unwrap()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatProtobuf::new(
            get_input_as_bytes(),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME.to_string(),
        )
        .unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn to_vec_u8() {
        let input = PayloadFormatProtobuf::new(
            get_input_as_bytes(),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME.to_string(),
        )
        .unwrap();

        let result: Vec<u8> = Vec::try_from(input).unwrap();

        assert_eq!(get_input_as_bytes(), result);
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::from("not possible");
        let result = PayloadFormatProtobuf::convert_from(
            PayloadFormat::Text(input),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME,
        );
        assert!(result.is_err());
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input_as_bytes()).unwrap();
        let result = PayloadFormatProtobuf::convert_from(
            PayloadFormat::Raw(input),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME,
        )
        .unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn from_hex() {
        let input = PayloadFormatHex::try_from(INPUT_STRING_HEX.to_owned()).unwrap();
        let result = PayloadFormatProtobuf::convert_from(
            PayloadFormat::Hex(input),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME,
        )
        .unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn from_base64() {
        let input = PayloadFormatBase64::try_from(INPUT_STRING_BASE64.to_owned()).unwrap();
        let result = PayloadFormatProtobuf::convert_from(
            PayloadFormat::Base64(input),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME,
        )
        .unwrap();

        assert_eq!(32, extract_distance(&result));
        assert_eq!("kindof".to_string(), extract_kind(&result));
    }

    #[test]
    fn from_yaml() {
        let input = PayloadFormatYaml::try_from(Vec::<u8>::from("content: input")).unwrap();
        let result = PayloadFormatProtobuf::convert_from(
            PayloadFormat::Yaml(input),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME,
        );
        assert!(result.is_err());
    }

    #[test]
    fn from_json() {
        let input =
            PayloadFormatJson::try_from(Vec::<u8>::from("{\"content\":\"input\"}")).unwrap();
        let result = PayloadFormatProtobuf::convert_from(
            PayloadFormat::Json(input),
            &INPUT_PATH_MESSAGE,
            MESSAGE_NAME,
        );
        assert!(result.is_err());
    }

    fn extract_kind(result: &PayloadFormatProtobuf) -> String {
        let descriptor = result.content.descriptor_dyn();

        let inside = descriptor.field_by_name("inside").unwrap();
        let inside_value = inside.get_singular(result.content().deref()).unwrap();

        let msg_inside = inside_value.to_message().unwrap();
        let kind = msg_inside.descriptor_dyn().field_by_name("kind").unwrap();

        let kind_value = kind.get_singular(msg_inside.deref()).unwrap();

        kind_value.to_string()
    }

    fn extract_distance(result: &PayloadFormatProtobuf) -> i32 {
        let descriptor = result.content.descriptor_dyn();

        let kind = descriptor.field_by_name("distance").unwrap();

        let kind_value = kind.get_singular(result.content().deref()).unwrap();
        kind_value.to_i32().unwrap()
    }
}

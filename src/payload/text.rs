use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatText {
    content: String,
}

pub type PayloadFormatTextInput = Vec<u8>;

impl TryFrom<PayloadFormatTextInput> for PayloadFormatText {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormatTextInput) -> Result<Self, Self::Error> {
        Ok(Self {
            content: String::from_utf8(value)?,
        })
    }
}

impl From<PayloadFormatText> for Vec<u8> {
    fn from(val: PayloadFormatText) -> Self {
        val.content.into_bytes()
    }
}

impl From<PayloadFormatText> for String {
    fn from(val: PayloadFormatText) -> Self {
        val.content
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatText {
    type Error = PayloadFormatError;

    fn try_from(value: PayloadFormat) -> Result<Self, Self::Error> {
        match value {
            PayloadFormat::Text(value) => Ok(value),
            PayloadFormat::Raw(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Protobuf(value) => Ok(Self {
                content: protobuf::get_message_value(
                    value.context(),
                    value.message_value(),
                    0,
                    None,
                )?,
            }),
            PayloadFormat::Hex(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Base64(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Json(value) => {
                let a: Vec<u8> = value.into();
                Self::try_from(a)
            }
            PayloadFormat::Yaml(value) => {
                let a: Vec<u8> = value.try_into()?;
                Self::try_from(a)
            }
        }
    }
}

mod protobuf {
    use protofish::context::{Context, MessageInfo};
    use protofish::decode::{FieldValue, MessageValue, Value};

    use crate::payload::PayloadFormatError;

    pub(super) fn get_message_value(
        context: &Context,
        message_value: &MessageValue,
        indent_level: u16,
        parent_field: Option<u64>,
    ) -> Result<String, PayloadFormatError> {
        let mut result = String::new();

        let message_info = context.resolve_message(message_value.msg_ref);

        let message_text = match parent_field {
            None => {
                format!("{}\n", message_info.full_name)
            }
            Some(parent_field) => {
                let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();
                format!(
                    "{indent_spaces}[{}] {}\n",
                    parent_field, message_info.full_name
                )
            }
        };
        result.push_str(&message_text);

        for field in &message_value.fields {
            let result_field = get_field_value(context, message_info, field, indent_level + 1)?;
            result.push_str(&result_field);
        }

        Ok(result)
    }

    fn get_field_value(
        context: &Context,
        message_response: &MessageInfo,
        field_value: &FieldValue,
        indent_level: u16,
    ) -> Result<String, PayloadFormatError> {
        let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();

        return match &message_response.get_field(field_value.number) {
            None => Err(PayloadFormatError::FieldNumberNotFoundInProtoFile(
                field_value.number,
            )),
            Some(field) => {
                let type_name = &field.name;

                let ret = match &field_value.value {
                    Value::Double(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Double)\n",
                            field.number, value
                        )
                    }
                    Value::Float(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Float)\n",
                            field.number, value
                        )
                    }
                    Value::Int32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Int32)\n",
                            field.number, value
                        )
                    }
                    Value::Int64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Int64)\n",
                            field.number, value
                        )
                    }
                    Value::UInt32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (UInt32)\n",
                            field.number, value
                        )
                    }
                    Value::UInt64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (UInt64)\n",
                            field.number, value
                        )
                    }
                    Value::SInt32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SInt32)\n",
                            field.number, value
                        )
                    }
                    Value::SInt64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SInt64)\n",
                            field.number, value
                        )
                    }
                    Value::Fixed32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Fixed32)\n",
                            field.number, value
                        )
                    }
                    Value::Fixed64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Fixed64)\n",
                            field.number, value
                        )
                    }
                    Value::SFixed32(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SFixed32)\n",
                            field.number, value
                        )
                    }
                    Value::SFixed64(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (SFixed64)\n",
                            field.number, value
                        )
                    }
                    Value::Bool(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (Bool)\n",
                            field.number, value
                        )
                    }
                    Value::String(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {} (String)\n",
                            field.number, value
                        )
                    }
                    Value::Bytes(value) => {
                        format!(
                            "{indent_spaces}[{}] {type_name} = {:?} (Bytes)\n",
                            field.number, value
                        )
                    }
                    Value::Message(value) => {
                        get_message_value(context, value, indent_level, Some(field.number))?
                    }
                    value => {
                        format!(
                            "{indent_spaces}[{}] Unknown value encountered: {:?}\n",
                            field.number, value
                        )
                    }
                };

                Ok(ret)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::payload::raw::PayloadFormatRaw;

    use super::*;

    const INPUT_STRING: &str = "INPUT";

    fn get_input() -> Vec<u8> {
        INPUT_STRING.into()
    }

    #[test]
    fn from_vec_u8() {
        let result = PayloadFormatText::try_from(get_input());

        assert_eq!(true, result.is_ok());
        assert_eq!(String::from(INPUT_STRING), result.unwrap().content);
    }

    #[test]
    fn to_vec_u8() {
        let input = PayloadFormatText::try_from(get_input());
        let result: Vec<u8> = input.unwrap().into();

        assert_eq!(INPUT_STRING.as_bytes(), result.as_slice());
    }

    #[test]
    fn from_text() {
        let input = PayloadFormatText::try_from(get_input());
        let result = PayloadFormatText::try_from(PayloadFormat::Text(input.unwrap()));

        assert_eq!(true, result.is_ok());
        assert_eq!(String::from(INPUT_STRING), result.unwrap().content);
    }

    #[test]
    fn from_raw() {
        let input = PayloadFormatRaw::try_from(get_input());
        let result = PayloadFormatText::try_from(PayloadFormat::Raw(input.unwrap()));

        assert_eq!(true, result.is_ok());
        assert_eq!(String::from(INPUT_STRING), result.unwrap().content);
    }
}

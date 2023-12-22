use base64::engine::general_purpose;
use base64::Engine;

use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatBase64 {
    content: Vec<u8>,
}

pub type PayloadFormatBase64Input = Vec<u8>;

impl From<PayloadFormatBase64Input> for PayloadFormatBase64 {
    fn from(value: PayloadFormatBase64Input) -> Self {
        Self { content: value }
    }
}

impl From<PayloadFormatBase64> for Vec<u8> {
    fn from(val: PayloadFormatBase64) -> Self {
        val.content
    }
}

impl From<PayloadFormatBase64> for String {
    fn from(val: PayloadFormatBase64) -> Self {
        general_purpose::STANDARD_NO_PAD.encode(val.content)
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
                let a: Vec<u8> = value.into();
                Ok(Self::from(a))
            }
            PayloadFormat::Yaml(value) => {
                let a: Vec<u8> = value.try_into()?;
                Ok(Self::from(a))
            }
        }
    }
}

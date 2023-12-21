use base64::Engine;
use base64::engine::general_purpose;
use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatBase64 {
    content: Vec<u8>,
}

pub type PayloadFormatBase64Input = Vec<u8>;

impl From<PayloadFormatBase64Input> for PayloadFormatBase64 {
    fn from(value: PayloadFormatBase64Input) -> Self {
        Self {
            content: value
        }
    }
}

impl Into<Vec<u8>> for PayloadFormatBase64 {
    fn into(self) -> Vec<u8> {
        self.content
    }
}

impl Into<String> for PayloadFormatBase64 {
    fn into(self) -> String {
        general_purpose::STANDARD_NO_PAD.encode(self.content)
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

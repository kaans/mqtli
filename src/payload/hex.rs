use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatHex {
    content: String,
}

pub type PayloadFormatHexInput = Vec<u8>;

impl From<PayloadFormatHexInput> for PayloadFormatHex {
    fn from(value: PayloadFormatHexInput) -> Self {
        Self {
            content: hex::encode_upper(value)
        }
    }
}

impl Into<Vec<u8>> for PayloadFormatHex {
    fn into(self) -> Vec<u8> {
        self.content.into_bytes()
    }
}

impl TryFrom<PayloadFormat> for PayloadFormatHex {
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
            PayloadFormat::Hex(value) => Ok(value),
            PayloadFormat::Base64(value) => {
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

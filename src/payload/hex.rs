use crate::payload::{PayloadFormat, PayloadFormatError};

#[derive(Clone, Debug)]
pub struct PayloadFormatHex {
    content: Vec<u8>,
}

impl PayloadFormatHex {
    pub fn decode_from(value: Vec<u8>) -> Result<Self, PayloadFormatError> {
        Ok(Self {
            content: hex::decode(value)?
        })
    }
}

impl From<Vec<u8>> for PayloadFormatHex {
    fn from(value: Vec<u8>) -> Self {
        Self {
            content: value
        }
    }
}

impl TryFrom<String> for PayloadFormatHex {
    type Error = PayloadFormatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            content: hex::decode(value)?
        })
    }
}

impl Into<Vec<u8>> for PayloadFormatHex {
    fn into(self) -> Vec<u8> {
        self.content
    }
}

impl Into<String> for PayloadFormatHex {
    fn into(self) -> String {
        hex::encode(self.content)
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

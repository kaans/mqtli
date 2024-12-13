use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::payload::PayloadFormatError;

pub mod console;
pub mod file;

#[derive(Error, Debug)]
pub enum OutputError {
    #[error("Could not open target file \"{1}\"")]
    CouldNotOpenTargetFile(#[source] io::Error, PathBuf),
    #[error("Error while writing to file \"{1}\"")]
    ErrorWhileWritingToFile(#[source] io::Error, PathBuf),
    #[error("Error while formatting payload: {0}")]
    ErrorPayloadFormat(#[source] PayloadFormatError),
}

impl From<PayloadFormatError> for OutputError {
    fn from(value: PayloadFormatError) -> Self {
        Self::ErrorPayloadFormat(value)
    }
}

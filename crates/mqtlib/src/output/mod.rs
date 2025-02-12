use std::io;
use std::path::PathBuf;

use crate::mqtt::MessageEvent;
use crate::payload::PayloadFormatError;
use crate::storage::SqlStorageError;
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;

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
    #[error("Error while sending payload to topic: {0}")]
    SendError(#[source] SendError<MessageEvent>),
    #[error("SQL database is not initialized")]
    SqlDatabaseNotInitialized,
    #[error("SQL Storage Error")]
    SqlStorageError(#[from] SqlStorageError),
}

impl From<PayloadFormatError> for OutputError {
    fn from(value: PayloadFormatError) -> Self {
        Self::ErrorPayloadFormat(value)
    }
}

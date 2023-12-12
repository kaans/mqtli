use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use thiserror::Error;

pub mod console;
pub mod file;

#[derive(Error, Debug)]
pub enum OutputError {
    #[error("Could not open target file \"{1}\"")]
    CouldNotOpenTargetFile(#[source] io::Error, PathBuf),
    #[error("Error while writing to file \"{1}\"")]
    ErrorWhileWritingToFile(#[source] io::Error, PathBuf),
    #[error("Could not decode UTF8")]
    CouldNotDecodeUtf8(#[source] FromUtf8Error),
}
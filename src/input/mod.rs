use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use protofish::context::ParseError;
use thiserror::Error;

pub mod converter;
mod text;
mod raw;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Cannot read content from path {1}")]
    CannotReadInputFromPath(#[source] io::Error, PathBuf),
    #[error("Either content or path to content must be given")]
    EitherContentOrPathMustBeGiven,
    #[error("Conversion from format {0} to format {1} not possible")]
    ConversionNotPossible(String, String),
    #[error("Could not decode UTF8")]
    CouldNotDecodeUtf8(#[source] FromUtf8Error),
    #[error("Could not open definition file {0}")]
    CouldNotOpenDefinitionFile(String),
    #[error("Could not parse proto file {0}")]
    CouldNotParseProtoFile(#[source] ParseError),
    #[error("Message {0} not found in proto file, cannot decode payload")]
    MessageNotFoundInProtoFile(String),
    #[error("Invalid protobuf")]
    InvalidProtobuf
}

fn read_from_path(path: &PathBuf) -> Result<Vec<u8>, InputError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { return Err(InputError::CannotReadInputFromPath(e, PathBuf::from(path))); }
    };

    let mut buf = Vec::new();
    if let Err(e) = file.read_to_end(&mut buf) {
        return Err(InputError::CannotReadInputFromPath(e, PathBuf::from(path)));
    };
    Ok(buf)
}
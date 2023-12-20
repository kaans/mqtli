use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub mod converter;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Cannot read content from path {1}")]
    CannotReadInputFromPath(#[source] io::Error, PathBuf),
    #[error("Either content or path to content must be given")]
    EitherContentOrPathMustBeGiven,
}

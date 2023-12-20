use std::fs::{File, read_to_string};
use std::io::Read;
use std::path::PathBuf;

use crate::config::mqtli_config::{PublishInputType, PublishInputTypeContentPath, PublishInputTypePath};
use crate::input::InputError;

pub struct InputConverter {}

impl InputConverter {
    pub fn convert_input(input: &PublishInputType) -> Result<Vec<u8>, InputError> {
        match input {
            PublishInputType::Text(input) => {
                Self::convert_from_text(input)
            }
            PublishInputType::Raw(input)
            => Self::convert_from_raw(input)
        }
    }

    fn convert_from_text(input: &PublishInputTypeContentPath) -> Result<Vec<u8>, InputError> {
        if let Some(content) = input.content() {
            Ok(Vec::from(content.as_str()))
        } else {
            if let Some(path) = input.path() {
                match read_to_string(path) {
                    Ok(content) => Ok(Vec::from(content)),
                    Err(e) => {
                        Err(InputError::CannotReadInputFromPath(e, PathBuf::from(path)))
                    }
                }
            } else {
                Err(InputError::EitherContentOrPathMustBeGiven)
            }
        }
    }

    fn convert_from_raw(input: &PublishInputTypePath) -> Result<Vec<u8>, InputError> {
        let mut file = match File::open(input.path()) {
            Ok(f) => f,
            Err(e) => { return Err(InputError::CannotReadInputFromPath(e, PathBuf::from(input.path()))); }
        };

        let mut buf = Vec::new();
        match file.read_to_end(&mut buf) {
            Ok(_size) => Ok(buf),
            Err(e) => Err(InputError::CannotReadInputFromPath(e, PathBuf::from(input.path())))
        }
    }
}
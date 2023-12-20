use std::fs::read_to_string;
use std::path::PathBuf;

use crate::config::mqtli_config::{PublishInputType, PublishInputTypeText};
use crate::input::InputError;

pub struct InputConverter {}

impl InputConverter {
    pub fn convert_input(input: &PublishInputType) -> Result<Vec<u8>, InputError> {
        match input {
            PublishInputType::Text(input) => {
                Self::convert_to_text(input)
            }
        }
    }

    fn convert_to_text(input: &PublishInputTypeText) -> Result<Vec<u8>, InputError> {
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

    fn convert_to_raw(input: &PublishInputTypeText) -> Result<Vec<u8>, InputError> {
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
}
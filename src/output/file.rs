use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::config::mqtli_config::OutputTargetFile;
use crate::output::OutputError;

pub struct FileOutput {}

impl FileOutput {
    pub fn output(content: Vec<u8>, target_file: &OutputTargetFile) -> Result<(), OutputError> {
        match File::options()
            .append(*target_file.append()).write(true).create(true).open(target_file.path()) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(content.as_slice()) {
                    return Err(OutputError::ErrorWhileWritingToFile(e, PathBuf::from(target_file.path())));
                }

                if *target_file.newline() {
                    if let Err(e) = file.write_all("\n".as_bytes()) {
                        return Err(OutputError::ErrorWhileWritingToFile(e, PathBuf::from(target_file.path())));
                    }
                }

                Ok(())
            }
            Err(e) => {
                Err(OutputError::CouldNotOpenTargetFile(e, PathBuf::from(target_file.path())))
            }
        }
    }
}
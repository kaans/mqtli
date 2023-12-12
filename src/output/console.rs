use crate::output::OutputError;

pub struct ConsoleOutput {}

impl ConsoleOutput {
    pub fn output(content: Vec<u8>) -> Result<(), OutputError> {
        match String::from_utf8(content) {
            Ok(content) => {
                println!("{}", content);
                Ok(())
            }
            Err(e) => {
                Err(OutputError::CouldNotDecodeUtf8(e))
            }
        }
    }
}
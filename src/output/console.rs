use crate::output::OutputError;

pub struct ConsoleOutput {}

impl ConsoleOutput {
    pub fn output(content: String) -> Result<(), OutputError> {
        println!("{}", content);
        Ok(())
    }
}
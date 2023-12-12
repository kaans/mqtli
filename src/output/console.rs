
pub struct ConsoleOutput {}

impl ConsoleOutput {
    pub fn output(content: Vec<u8>) {
        println!("{}", String::from_utf8(content).unwrap_or("invalid content".to_string()));
    }
}
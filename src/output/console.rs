use crate::mqtt::QoS;
use crate::output::OutputError;
use colored::Colorize;
use crate::payload::PayloadFormat;

pub struct ConsoleOutput {}

impl ConsoleOutput {
    pub fn output(topic: &str, content: String, format: PayloadFormat, qos: QoS, retain: bool) -> Result<(), OutputError> {
        let retained = if retain { " retained" } else { "" };
        let bytes = if content.len() == 1 { "byte" } else { "bytes" };

        println!(
            "{} [{} | {} {} | {}] {}",
            topic.bold().green(),
            format.to_string().blue(),
            content.len().to_string().blue(),
            bytes.blue(),
            qos.to_string().blue(),
            retained.purple()
        );
        println!("{}", content.yellow());
        Ok(())
    }
}

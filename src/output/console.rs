use crate::mqtt::QoS;
use crate::output::OutputError;
use colored::Colorize;

pub struct ConsoleOutput {}

impl ConsoleOutput {
    pub fn output(topic: &str, content: String, qos: QoS, retain: bool) -> Result<(), OutputError> {
        let retained = if retain { " retained" } else { "" };
        println!(
            "{} [{}] {}",
            topic.bold().green(),
            qos.to_string().blue(),
            retained.purple()
        );
        println!("{}", content.yellow());
        Ok(())
    }
}

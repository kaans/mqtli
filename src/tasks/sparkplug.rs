use chrono::DateTime;
use colored::Colorize;
use mqtlib::config::subscription::OutputTarget;
use mqtlib::config::topic::TopicStorage;
use mqtlib::mqtt::MessageEvent;
use mqtlib::output::console::ConsoleOutput;
use mqtlib::output::file::FileOutput;
use mqtlib::payload::sparkplug::protos::sparkplug_b::payload::metric::Value;
use mqtlib::payload::sparkplug::PayloadFormatSparkplug;
use mqtlib::payload::PayloadFormat;
use mqtlib::sparkplug::{SparkplugMessageType, SparkplugNetwork, SparkplugTopic, SparkplugTopicEdgeNode};
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;
use tracing::error;
use mqtlib::payload::sparkplug::protos::sparkplug_b::payload::Metric;

pub fn start_sparkplug_monitor(
    sparkplug_network: Arc<Mutex<SparkplugNetwork>>,
    topic_storage: Arc<TopicStorage>,
    mut receiver: Receiver<MessageEvent>,
) {
    tracing::debug!("Starting sparkplug network monitor");

    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(MessageEvent::ReceivedUnfiltered(message)) => {
                    if let PayloadFormat::Sparkplug(payload) = message.payload {
                        tracing::debug!("Received sparkplug message on topic {}", message.topic);
                        tracing::trace!("{}", payload);

                        match SparkplugTopic::try_from(message.topic) {
                            Ok(topic) => {
                                output_sparkplug_message(&payload, &topic, topic_storage.clone());

                                sparkplug_network.lock().await.parse_message(topic, payload);
                            }
                            Err(e) => {
                                error!("Error while parsing sparkplug topic: {e:?}");
                            }
                        };
                    }
                }
                Err(RecvError::Lagged(skipped_messages)) => {
                    tracing::warn!("Receiver skipped {skipped_messages} messages");
                }
                Err(RecvError::Closed) => break,
                _ => {}
            }
        }

        tracing::debug!("Sparkplug network monitor exited");
    });
}

fn output_sparkplug_message(
    message: &PayloadFormatSparkplug,
    topic: &SparkplugTopic,
    topic_storage: Arc<TopicStorage>,
) {
    let outputs = topic_storage.get_outputs_for_topic(topic.to_string().as_str());

    let content: String = match topic {
        SparkplugTopic::EdgeNode(topic) => match topic.message_type {
            SparkplugMessageType::NBIRTH => {
                format_nbirth(message, topic)
            }
            SparkplugMessageType::NDATA => {
                format_ndata(message, topic)
            }
            SparkplugMessageType::NDEATH => {
                let mut result: Vec<String> = vec![];
                result.push(format!(
                    "Edge node \"{}\" left the network",
                    topic.edge_node_id
                ));

                result
            }
            SparkplugMessageType::DBIRTH => {
                format_dbirth(message, topic)
            }
            SparkplugMessageType::DDATA => {
                format_ddata(message, topic)
            }
            SparkplugMessageType::DDEATH => {
                let mut result: Vec<String> = vec![];
                result.push(format!(
                    "Device \"{}\" left the network",
                    topic.device_id.as_ref().unwrap_or(&"unknown".to_string())
                ));

                result
            }
            SparkplugMessageType::NCMD => {
                let mut result: Vec<String> = vec![];
                result.push(format!(
                    "Command arrived from edge node \"{}\"",
                    topic.edge_node_id
                ));

                result
            }
            SparkplugMessageType::DCMD => {
                let mut result: Vec<String> = vec![];
                result.push(format!(
                    "Command arrived from device \"{}\"",
                    topic.device_id.as_ref().unwrap_or(&"unknown".to_string())
                ));

                result
            }
            _ => vec![],
        },
        SparkplugTopic::HostApplication(topic) => match topic.message_type {
            SparkplugMessageType::STATE => {
                let mut result: Vec<String> = vec![];
                result.push(format!(
                    "State arrived from host application \"{}\"",
                    topic.host_id
                ));

                result
            }
            _ => vec![],
        },
    }
        .join("\n");

    for output in outputs {
        if let Err(e) = match output.target() {
            OutputTarget::Console(_options) => ConsoleOutput::output_string(content.clone()),
            OutputTarget::File(file) => FileOutput::output(content.clone().into_bytes(), file),
            _ => Ok(()),
        } {
            error!("Error while printing sparkplug message: {e:?}");
        }
    }
}

fn format_ddata(message: &PayloadFormatSparkplug, topic: &SparkplugTopicEdgeNode) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let content = format!(
        "[{}] {}/{}/{} (seq {})",
        topic.message_type.to_string().blue(),
        topic.group_id.yellow(),
        topic.edge_node_id.magenta(),
        topic
            .device_id
            .as_ref()
            .unwrap_or(&"unknown".to_string())
            .blue(),
        message.content.seq.unwrap_or(999).to_string().white()
    )
        .black().on_cyan();

    result.push(content.to_string());
    result.extend(add_metrics(&message.content.metrics, false));

    result
}


fn format_ndata(message: &PayloadFormatSparkplug, topic: &SparkplugTopicEdgeNode) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let content = format!(
        "[{}] {}/{} (seq {})",
        topic.message_type.to_string().magenta(),
        topic.group_id.yellow(),
        topic.edge_node_id.magenta(),
        message.content.seq.unwrap_or(999).to_string().white()
    )
        .black().on_cyan();

    result.push(content.to_string());
    result.extend(add_metrics(&message.content.metrics, false));

    result
}

fn format_nbirth(message: &PayloadFormatSparkplug, topic: &SparkplugTopicEdgeNode) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let content = format!(
        "[{}] Edge node {}/{} joined the network (seq {})",
        topic.message_type.to_string().magenta(),
        topic.group_id.yellow(),
        topic.edge_node_id.magenta(),
        message.content.seq.unwrap_or(999).to_string().white()
    )
        .black().on_cyan();

    result.push(content.to_string());
    result.extend(add_metrics(&message.content.metrics, false));

    result
}

fn format_dbirth(message: &PayloadFormatSparkplug, topic: &SparkplugTopicEdgeNode) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let content = format!(
        "[{}] Device node {}/{}/{} joined the network (seq {})",
        topic.message_type.to_string().blue(),
        topic.group_id.yellow(),
        topic.edge_node_id.magenta(),
        topic
            .device_id
            .as_ref()
            .unwrap_or(&"unknown".to_string())
            .blue(),
        message.content.seq.unwrap_or(999).to_string().white()
    )
        .black().on_cyan();

    result.push(content.to_string());
    result.extend(add_metrics(&message.content.metrics, false));

    result
}

fn add_metrics(metrics: &Vec<Metric>, is_template: bool) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    for metric in metrics {
        let value = if metric.is_null() {
            "null".to_string()
        } else {
            match &metric.value {
                None => "unknown".to_string(),
                Some(value) => match value {
                    Value::IntValue(value) => format!("{}", value),
                    Value::LongValue(value) => format!("{}", value),
                    Value::FloatValue(value) => format!("{}", value),
                    Value::DoubleValue(value) => format!("{}", value),
                    Value::BooleanValue(value) => format!("{}", value),
                    Value::StringValue(value) => value.clone(),
                    Value::BytesValue(value) => {
                        format!("{}", String::from_utf8_lossy(value.as_ref()))
                    }
                    Value::DatasetValue(value) => format!("{}", value),
                    Value::TemplateValue(value) => {
                        format!("Template\n{}", add_metrics(&value.metrics, true).join("\n"))
                    }
                    Value::ExtensionValue(value) => format!("{}", value),
                    &_ => "".to_string(),
                },
            }
        };

        let data = match is_template {
            true => {
                format!(
                    "    [{}{}{}] {} = {}",
                    metric
                        .timestamp
                        .map_or("unknown".to_string(), |t| {
                            if let Some(utc) = DateTime::from_timestamp_millis(t as i64) {
                                return utc.format("%H:%M:%S%.3f").to_string();
                            }
                            "unknown".to_string()
                        }),
                    if metric.is_historical() { ", historical".red().to_string() } else { "".to_string() },
                    if metric.is_transient() { ", transient".red().to_string() } else { "".to_string() },
                    metric.name.clone().unwrap_or("unknown".to_string()).green(),
                    value,
                )
            }
            false => {
                format!(
                    "- [{}{}{}] {} = {}",
                    metric
                        .timestamp
                        .map_or("unknown".to_string(), |t| {
                            if let Some(utc) = DateTime::from_timestamp_millis(t as i64) {
                                return utc.format("%H:%M:%S%.3f").to_string();
                            }
                            "unknown".to_string()
                        }),
                    if metric.is_historical() { ", historical".red().to_string() } else { "".to_string() },
                    if metric.is_transient() { ", transient".red().to_string() } else { "".to_string() },
                    metric.name.clone().unwrap_or("unknown".to_string()).green(),
                    value,
                )
            }
        };
        result.push(data.white().to_string());
    }

    result
}

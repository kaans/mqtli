use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::str::{from_utf8, Utf8Error};

use log::{debug, error, info};
use protofish::context::{Context, MessageField, MessageInfo, TypeRef};
use protofish::decode::{FieldValue, Value};
use rumqttc::v5::{Event, Incoming};
use rumqttc::v5::mqttbytes::v5::Publish;
use tokio::sync::broadcast::Receiver;
use tokio::task;
use tokio::task::JoinHandle;
use crate::config::mqtli_config::{PayloadType, Topic};

pub struct MqttHandler {
    task_handle: Option<JoinHandle<()>>,
    subscribe_topics: Vec<Topic>,
}

impl MqttHandler {
    pub fn new(subscribe_topics: &Vec<Topic>) -> MqttHandler {
        let mut handler = MqttHandler {
            task_handle: None,
            subscribe_topics: subscribe_topics.clone(),
        };

        handler
    }

    pub(crate) fn start_task(&mut self, mut receiver: Receiver<Event>) {
        let subscribe_topics = self.subscribe_topics.clone();

        self.task_handle = Some(task::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        MqttHandler::handle_event(event, &subscribe_topics);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }));
    }

    pub async fn await_task(self) {
        if self.task_handle.is_some() {
            self.task_handle.unwrap().await.expect("Could not join thread");
        }
    }

    pub fn handle_event(event: Event, subscribe_topics: &Vec<Topic>) {
        match event {
            Event::Incoming(event) => {
                match event {
                    Incoming::Publish(value) => {
                        let incoming_topic = from_utf8(value.topic.as_ref()).unwrap();

                        info!("Handler {} (QoS: {:?})",
                                            incoming_topic,
                                            value.qos);

                        for topic in subscribe_topics {
                            if topic.topic() == incoming_topic {
                                debug!("Handling topic {}", incoming_topic);

                                match topic.payload() {
                                    PayloadType::Text(_) => {
                                        MqttHandler::handle_publish_text(&value);
                                    }
                                    PayloadType::Protobuf(payload) => {
                                        MqttHandler::handle_publish_protobuf(&value, payload.definition(), payload.message());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Outgoing(_event) => {}
        }
    }

    fn handle_publish_text(value: &Publish) {
        match from_utf8(value.payload.as_ref()) {
            Ok(content) => {
                println!("{}", content);
            }
            Err(e) => {
                error!("Could not convert payload to UTF 8 string: {e:?}");
            }
        }
    }

    fn handle_publish_protobuf(value: &Publish, definition_file: &PathBuf, message_name: &String) {
        let Ok(content) = read_to_string(definition_file) else {
            error!("Could not open definition file {definition_file:?}");
            return;
        };

        let context = match Context::parse(vec![content]) {
            Ok(context) => context,
            Err(e) => {
                error!("Could not parse proto file: {e:?}");
                return;
            }
        };

        let Some(message_response) = context.get_message("Proto.Response") else {
            error!("Message \"Proto.Response\" not found in proto file, cannot decode payload");
            return;
        };

        let dec = message_response.decode(value.payload.as_ref(), &context);

        for inner_type in &message_response.inner_types {
            match inner_type {
                TypeRef::Message(value) => {
                    let v = context.resolve_message(*value);
                    //println!("Message {:?}", v.full_name);
                }
                TypeRef::Enum(value) => {
                    let v = context.resolve_enum(*value);
                    //println!("Enum {:?}", v.full_name);
                }
            }
        }

        for field in dec.fields {
            if let Ok(result) = Self::get_field_value(&context, &message_response, &field, 0) {
                println!("{}", result);
            }
        }
    }

    fn get_field_value(context: &Context, message_response: &MessageInfo, field_value: &FieldValue, indent_level: u16) -> Result<String, ()> {
        let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();

        return match &message_response.get_field(field_value.number) {
            None => {
                error!("Field with number {} not found in message", field_value.number);

                Err(())
            },
            Some(field) => {
                let type_name = &field.name;

                let ret = match &field_value.value {
                    Value::Double(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Double)", field.number, value.to_string())
                    }
                    Value::Float(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Float)", field.number, value.to_string())
                    }
                    Value::Int32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Int32)", field.number, value.to_string())
                    }
                    Value::Int64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Int64)", field.number, value.to_string())
                    }
                    Value::UInt32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (UInt32)", field.number, value.to_string())
                    }
                    Value::UInt64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (UInt64)", field.number, value.to_string())
                    }
                    Value::SInt32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SInt32)", field.number, value.to_string())
                    }
                    Value::SInt64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SInt64)", field.number, value.to_string())
                    }
                    Value::Fixed32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Fixed32)", field.number, value.to_string())
                    }
                    Value::Fixed64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Fixed64)", field.number, value.to_string())
                    }
                    Value::SFixed32(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SFixed32)", field.number, value.to_string())
                    }
                    Value::SFixed64(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (SFixed64)", field.number, value.to_string())
                    }
                    Value::Bool(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (Bool)", field.number, value.to_string())
                    }
                    Value::String(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {} (String)", field.number, value)
                    }
                    Value::Bytes(value) => {
                        format!("{indent_spaces}[{}] {type_name} = {:?} (Bytes)", field.number, value)
                    }
                    Value::Message(value) => {
                        let message_inner = context.resolve_message(value.msg_ref);
                        let mut ret = format!("[{}] {}", field.number, &message_inner.full_name);

                        for field in &value.fields {
                            let ret_inner = Self::get_field_value(context, message_inner, &field, indent_level + 1)?;

                            ret.push_str(format!("\n{}", ret_inner).as_str());
                        }

                        ret
                    }
                    value => {
                        "Unknown".to_string()
                    }
                };

                Ok(ret)
            }
        }
    }
}
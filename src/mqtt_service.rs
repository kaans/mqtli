use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use std::str::{from_utf8, Utf8Error};
use std::sync::Arc;

use log::{debug, error, info};
use protofish::context::{Context, MessageInfo, TypeRef};
use protofish::decode::{FieldValue, Value};
use rumqttc::{Key, TlsConfiguration, Transport};
use rumqttc::v5::{AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions};
use rumqttc::v5::mqttbytes::v5::ConnectReturnCode;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::{MqttBrokerConnectArgs, Topic};

#[derive(Error, Debug)]
pub enum MqttServiceError {
    #[error("CA certificate must be present when using TLS")]
    CaCertificateMustBePresent(),
    #[error("Could not read CA certificate from file \"{1}\"")]
    CaCertificateNotReadable(#[source] io::Error, PathBuf),
    #[error("Could not read client certificate from file \"{1}\"")]
    ClientCertificateNotReadable(#[source] io::Error, PathBuf),
    #[error("Could not read client key from file \"{1}\"")]
    ClientKeyNotReadable(#[source] io::Error, PathBuf),
    #[error("Invalid client key in file \"{1}\"")]
    InvalidClientKey(#[source] Utf8Error, PathBuf),
    #[error("The client key must be either RSA or ECC in file \"{0}\"")]
    UnsupportedClientKey(PathBuf),
}

pub struct MqttService<'a> {
    client: Option<AsyncClient>,
    config: &'a MqttBrokerConnectArgs,

    subscribe_topics: Arc<Mutex<Vec<Topic>>>,

    task_handle: Option<JoinHandle<()>>,
}

impl MqttService<'_> {
    pub fn new(mqtt_connect_args: &MqttBrokerConnectArgs) -> MqttService {
        MqttService {
            client: None,
            config: mqtt_connect_args,

            subscribe_topics: Arc::new(Mutex::new(vec![])),
            task_handle: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), MqttServiceError> {
        info!("Connection to {}:{} with client id {}", self.config.host(),
            self.config.port(), self.config.client_id());
        let mut options = MqttOptions::new(self.config.client_id(),
                                           self.config.host(),
                                           *self.config.port());

        if *self.config.use_tls() {
            info!("Using TLS");

            let ca: Vec<u8> = match &self.config.tls_ca_file() {
                Some(ca_file) => match read_to_string(ca_file) {
                    Ok(ca) => ca.into_bytes(),
                    Err(e) => return Err(MqttServiceError::CaCertificateNotReadable(
                        e, PathBuf::from(ca_file))),
                },
                None => {
                    return Err(MqttServiceError::CaCertificateMustBePresent());
                }
            };

            let client_auth: Option<(Vec<u8>, Key)> = match self.config.tls_client_certificate() {
                None => None,
                Some(cert_file) => {
                    info!("Using TLS client certificate authentication");

                    let cert = match read_to_string(cert_file) {
                        Ok(ca) => ca.into_bytes(),
                        Err(e) => return Err(MqttServiceError::ClientCertificateNotReadable(
                            e, PathBuf::from(cert_file))),
                    };

                    let client_key_file = self.config.tls_client_key().as_ref().unwrap();
                    let key = match read_to_string(client_key_file) {
                        Ok(ca) => ca.into_bytes(),
                        Err(e) => return Err(MqttServiceError::ClientKeyNotReadable(
                            e, PathBuf::from(client_key_file))),
                    };

                    let key: Key = match from_utf8(key.as_slice()) {
                        Ok(content) => {
                            if content.contains("-----BEGIN RSA PRIVATE KEY-----") {
                                Key::RSA(key)
                            } else if content.contains("-----BEGIN EC PRIVATE KEY-----") {
                                Key::ECC(key)
                            } else {
                                return Err(MqttServiceError::UnsupportedClientKey(
                                    PathBuf::from(client_key_file)));
                            }
                        }
                        Err(e) => return Err(MqttServiceError::InvalidClientKey(
                            e, PathBuf::from(client_key_file))),
                    };

                    Some((cert, key))
                }
            };

            let tls_config = TlsConfiguration::Simple {
                ca,
                alpn: None,
                client_auth,
            };

            options.set_transport(Transport::Tls(tls_config));
        }

        debug!("Setting keep alive to {} seconds", self.config.keep_alive().as_secs());
        options.set_keep_alive(*self.config.keep_alive());

        if self.config.username().is_some() && self.config.password().is_some() {
            info!("Using username/password for authentication");
            options.set_credentials(self.config.username().clone().unwrap(),
                                    self.config.password().clone().unwrap());
        } else {
            info!("Using anonymous access");
        }

        let (client, event_loop) = AsyncClient::new(options, 10);
        self.client = Option::from(client.clone());

        let subscribe_topics = self.subscribe_topics.clone();

        let task_handle: JoinHandle<()> = MqttService::start_connection_task(event_loop,
                                                                             client,
                                                                             subscribe_topics)
            .await;

        self.task_handle = Some(task_handle);

        Ok(())
    }

    async fn start_connection_task(mut event_loop: EventLoop,
                                   client: AsyncClient,
                                   subscribe_topics: Arc<Mutex<Vec<Topic>>>) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(event) => {
                        debug!("Received {:?}", event);

                        match event {
                            Event::Incoming(event) => {
                                match event {
                                    Incoming::ConnAck(_) => {
                                        info!("Connected to broker");

                                        for topic in subscribe_topics.lock().await.iter() {
                                            client.subscribe(topic.topic(), *topic.qos()).await.expect("could not subscribe");
                                        }
                                    }
                                    Incoming::Publish(v) => {
                                        info!("{} ({:?})-> {}",
                                            from_utf8(v.topic.as_ref()).unwrap(),
                                            v.qos,
                                            from_utf8(v.payload.as_ref()).unwrap());

                                        let context = match Context::parse(&[r#"
  syntax = "proto3";
  package Proto;

  message Inner { string kind = 1; }
  message Response { int32 distance = 1; Inner inner_type = 2; }
"#]) {
                                            Ok(context) => context,
                                            Err(e) => {
                                                error!("Could not parse proto file: {e:?}");
                                                continue;
                                            }
                                        };

                                        let Some(message_response) = context.get_message("Proto.Response") else {
                                            error!("Message \"Proto.Response\" not found in proto file, cannot decode payload");
                                            continue;
                                        };

                                        let dec = message_response.decode(v.payload.as_ref(), &context);

                                        //println!("Message: {:?}", message_response);

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
                                            let ret= Self::get_field_value(&context, &message_response, &field, 0);

                                            println!("{}", ret);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Event::Outgoing(_event) => {}
                        }
                    }
                    Err(e) => {
                        match e {
                            ConnectionError::ConnectionRefused(ConnectReturnCode::NotAuthorized) => {
                                error!("Not authorized, check if the credentials are valid");
                                return;
                            }
                            _ => {
                                error!("Error while processing mqtt loop: {:?}", e);
                                return;
                            }
                        }
                    }
                }
            }
        })
    }

    fn get_field_value(context: &Context, message_response: &MessageInfo, field: &FieldValue, indent_level: u16) -> String {
        let indent_spaces = (0..indent_level).map(|_| "  ").collect::<String>();

        let type_name = &message_response.get_field(field.number).unwrap().name;

        let ret = match &field.value {
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
                    let ret_inner = Self::get_field_value(context, message_inner, &field, indent_level + 1);

                    ret.push_str(format!("\n{}", ret_inner).as_str());
                }

                ret
            },
            value => {
                "Unknown".to_string()
            }
        };

        ret
    }

    pub async fn subscribe(&mut self, topic: Topic) {
        info!("Subscribing to topic {} with QoS {:?}", topic.topic(), topic.qos());

        self.subscribe_topics.lock().await.push(topic);
    }

    pub async fn await_task(self) {
        if self.task_handle.is_some() {
            self.task_handle.unwrap().await.expect("Could not join thread");
        }
    }
}
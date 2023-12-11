use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use std::str::{from_utf8, Utf8Error};
use std::sync::Arc;

use log::{debug, error, info, warn};
use rumqttc::{Key, TlsConfiguration, Transport};
use rumqttc::v5::{AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions};
use rumqttc::v5::mqttbytes::v5::ConnectReturnCode;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
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

    topics: Arc<Mutex<Vec<Topic>>>,

    task_handle: Option<JoinHandle<()>>,
}

impl MqttService<'_> {
    pub fn new(mqtt_connect_args: &MqttBrokerConnectArgs) -> MqttService {
        MqttService {
            client: None,
            config: mqtt_connect_args,

            topics: Arc::new(Mutex::new(vec![])),
            task_handle: None,
        }
    }

    pub async fn connect(&mut self, channel: Option<broadcast::Sender<Event>>) -> Result<(), MqttServiceError> {
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

        let topics = self.topics.clone();

        let task_handle: JoinHandle<()> =
            MqttService::start_connection_task(event_loop, client, topics, channel)
                .await;

        self.task_handle = Some(task_handle);

        Ok(())
    }

    async fn start_connection_task(mut event_loop: EventLoop,
                                   client: AsyncClient,
                                   topics: Arc<Mutex<Vec<Topic>>>,
                                   channel: Option<broadcast::Sender<Event>>) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(event) => {
                        debug!("Received {:?}", &event);

                        match &event {
                            Event::Incoming(event) => {
                                match event {
                                    Incoming::ConnAck(_) => {
                                        info!("Connected to broker");

                                        for topic in topics.lock().await.iter() {
                                            if *topic.subscription().enabled() {
                                                client.subscribe(topic.topic(), *topic.subscription().qos()).await.expect("could not subscribe");
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Event::Outgoing(_event) => {}
                        }

                        if channel.is_some() {
                            let _ = channel.as_ref().unwrap().send(event);
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

    pub async fn subscribe(&mut self, topic: Topic) {
        if *topic.subscription().enabled() {
            info!("Subscribing to topic {} with QoS {:?}", topic.topic(), topic.subscription().qos());
            self.topics.lock().await.push(topic);
        } else {
            warn!("Not subscribing to topic, not enabled :{}", topic.topic());
        }
    }

    pub async fn await_task(self) {
        if self.task_handle.is_some() {
            self.task_handle.unwrap().await.expect("Could not join thread");
        }
    }
}
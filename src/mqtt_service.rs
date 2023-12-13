use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, error, info, warn};
use rumqttc::{TlsConfiguration, Transport};
use rumqttc::tokio_rustls::rustls;
use rumqttc::tokio_rustls::rustls::{Certificate, PrivateKey};
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
    CertificateNotReadable(#[source] io::Error, PathBuf),
    #[error("Could not add CA certificate to root store")]
    CaCertificateNotAdded(#[source] rustls::Error),
    #[error("Could not read client key from file \"{1}\"")]
    PrivateKeyNotReadable(#[source] io::Error, PathBuf),
    #[error("No PKCS8-encoded private key found in file \"{0}\"")]
    PrivateKeyNoneFound(PathBuf),
    #[error("More than one PKCS8-encoded private key found in file \"{0}\"")]
    PrivateKeyTooManyFound(PathBuf),
    #[error("Client key must be present when using TLS authentication")]
    ClientKeyMustBePresent(),
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

            let tls_config_rustls = match self.configure_tls_rustls() {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

            options.set_transport(Transport::Tls(tls_config_rustls));
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

    fn configure_tls_rustls(&mut self) -> Result<TlsConfiguration, MqttServiceError> {
        fn load_private_key_from_file(path: &PathBuf) -> Result<PrivateKey, MqttServiceError> {
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => return Err(MqttServiceError::PrivateKeyNotReadable(e, PathBuf::from(path)))
            };
            let mut reader = BufReader::new(file);
            let mut keys = match rustls_pemfile::pkcs8_private_keys(&mut reader) {
                Ok(keys) => keys,
                Err(e) => return Err(MqttServiceError::PrivateKeyNotReadable(e, PathBuf::from(path)))
            };

            match keys.len() {
                0 => Err(MqttServiceError::PrivateKeyNoneFound(PathBuf::from(path))),
                1 => Ok(PrivateKey(keys.remove(0))),
                _ => Err(MqttServiceError::PrivateKeyTooManyFound(PathBuf::from(path))),
            }
        }

        fn load_certificates_from_file(path: &PathBuf) -> Result<Vec<Certificate>, MqttServiceError> {
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => return Err(MqttServiceError::CertificateNotReadable(e, PathBuf::from(path)))
            };
            let mut reader = BufReader::new(file);
            let certs = match rustls_pemfile::certs(&mut reader) {
                Ok(certs) => certs,
                Err(e) => return Err(MqttServiceError::CertificateNotReadable(e, PathBuf::from(path)))
            };

            Ok(certs.into_iter().map(Certificate).collect())
        }

        let mut root_store = rustls::RootCertStore::empty();

        match &self.config.tls_ca_file() {
            Some(ca_file) => {
                let certificates
                    = load_certificates_from_file(ca_file)?;

                info!("Found {} root ca certificates", certificates.len());

                for certificate in certificates {
                    if let Err(e) = root_store.add(&certificate) {
                        return Err(MqttServiceError::CaCertificateNotAdded(e));
                    }
                }
            }
            None => {
                return Err(MqttServiceError::CaCertificateMustBePresent());
            }
        };


        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store);

        let config = match self.config.tls_client_certificate() {
            None => {
                config.with_no_client_auth()
            }
            Some(client_certificate_file) => {
                info!("Using TLS client certificate authentication");

                let client_certificate =
                    load_certificates_from_file(client_certificate_file)?;

                let Some(client_key_file) = self.config.tls_client_key() else {
                    return Err(MqttServiceError::ClientKeyMustBePresent());
                };

                let client_key = load_private_key_from_file(client_key_file)?;

                config.with_client_auth_cert(client_certificate, client_key)
                    .unwrap()
            }
        };

        let tls_config = TlsConfiguration::Rustls {
            0: Arc::new(config),
        };

        Ok(tls_config)
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
                                                info!("Subscribing to topic {} with QoS {:?}", topic.topic(), topic.subscription().qos());
                                                if let Err(e) = client.subscribe(topic.topic(), *topic.subscription().qos()).await {
                                                    error!("Could not subscribe to topic {}: {}", topic.topic(), e);
                                                }
                                            } else {
                                                warn!("Not subscribing to topic, not enabled :{}", topic.topic());
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
            self.topics.lock().await.push(topic);
        }
    }

    pub async fn await_task(self) {
        if self.task_handle.is_some() {
            self.task_handle.unwrap().await.expect("Could not join thread");
        }
    }
}
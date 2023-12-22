use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, error, info};
use rumqttc::{TlsConfiguration, Transport};
use rumqttc::tokio_rustls::rustls;
use rumqttc::tokio_rustls::rustls::{Certificate, PrivateKey};
use rumqttc::v5::{AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::mqttbytes::v5::{ConnectReturnCode, LastWill};
use rustls::SupportedProtocolVersion;
use rustls::version::{TLS12, TLS13};
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::{MqttBrokerConnectArgs, TlsVersion};

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

pub struct MqttService {
    client: Option<AsyncClient>,
    config: Arc<MqttBrokerConnectArgs>,

    topics: Arc<Mutex<Vec<(String, QoS)>>>,

    receiver: Arc<Mutex<Receiver<i32>>>,
}

impl MqttService {
    pub fn new(mqtt_connect_args: Arc<MqttBrokerConnectArgs>, receiver_exit: Receiver<i32>) -> MqttService {
        MqttService {
            client: None,
            config: mqtt_connect_args,

            topics: Arc::new(Mutex::new(vec![])),
            receiver: Arc::new(Mutex::new(receiver_exit)),
        }
    }

    pub async fn _disconnect(&self) {
        if let Some(client) = self.client.as_ref() {
            match client.disconnect().await {
                Ok(_) => {
                    info!("Successfully disconnected");
                }
                Err(e) => {
                    error!("Error during disconnect: {:?}", e);
                }
            };
        }
    }

    pub async fn connect(&mut self, channel: Option<broadcast::Sender<Event>>) -> Result<JoinHandle<()>, MqttServiceError> {
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

        if let Some(last_will) = self.config.last_will() {
            info!("Setting last will for topic {} [Payload length: {}, QoS {:?}; retain: {}]",
                last_will.topic(),
                last_will.payload().len(),
                last_will.qos(),
                last_will.retain(),
            );
            let last_will = LastWill::new(
                last_will.topic(),
                last_will.payload().clone(),
                *last_will.qos(),
                *last_will.retain(),
                None,
            );
            options.set_last_will(last_will);
        }

        let (client, event_loop) = AsyncClient::new(options, 10);

        let topics = self.topics.clone();

        let task_handle: JoinHandle<()> =
            MqttService::start_connection_task(event_loop, client.clone(), topics, channel)
                .await;

        self.client = Option::from(client);

        self.start_exit_task().await;

        Ok(task_handle)
    }

    fn configure_tls_rustls(&mut self) -> Result<TlsConfiguration, MqttServiceError> {
        fn load_private_key_from_file(path: &PathBuf) -> Result<PrivateKey, MqttServiceError> {
            let file = match File::open(path) {
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
            let file = match File::open(path) {
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
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups();

        let pr: Vec<&'static SupportedProtocolVersion> = match self.config.tls_version() {
            TlsVersion::All => {
                debug!("Using TLS versions 1.2 and 1.3");
                vec![&TLS12, &TLS13]
            }
            TlsVersion::Version1_2 => {
                debug!("Using TLS version 1.2");
                vec![&TLS12]
            }
            TlsVersion::Version1_3 => {
                debug!("Using TLS version 1.3");
                vec![&TLS13]
            }
        };

        let config =
            config.with_protocol_versions(pr.as_slice()).unwrap()
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

        Ok(TlsConfiguration::Rustls(Arc::new(config)))
    }

    async fn start_connection_task(mut event_loop: EventLoop,
                                   client: AsyncClient,
                                   topics: Arc<Mutex<Vec<(String, QoS)>>>,
                                   channel: Option<broadcast::Sender<Event>>) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(event) => {
                        debug!("Received {:?}", &event);

                        match &event {
                            Event::Incoming(event) => {
                                if let Incoming::ConnAck(_) = event {
                                    info!("Connected to broker");

                                    for (topic, qos) in topics.lock().await.iter() {
                                        info!("Subscribing to topic {} with QoS {:?}", topic, qos);
                                        if let Err(e) = client.subscribe(topic, *qos).await {
                                            error!("Could not subscribe to topic {}: {}", topic, e);
                                        }
                                    }
                                }
                            }
                            Event::Outgoing(_event) => {}
                        }

                        if let Some(channel) = &channel {
                            let _ = channel.send(event);
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

    async fn start_exit_task(&mut self) -> JoinHandle<()> {
        let rec = self.receiver.clone();
        let client = self.client.clone().unwrap().clone();

        tokio::task::spawn(async move {
            match rec.lock().await.recv().await {
                Ok(_event) => {
                    match client.disconnect().await {
                        Ok(_) => {
                            info!("Successfully disconnected");
                        }
                        Err(e) => {
                            error!("Error during disconnect: {:?}", e);
                        }
                    };
                }
                Err(_) => {
                    // ignore
                }
            }
        })
    }

    pub async fn subscribe(&mut self, topic: String, qos: QoS) {
        self.topics.lock().await.push((topic, qos));
    }

    pub async fn publish(&self, topic: String, qos: QoS, retain: bool, payload: Vec<u8>) {
        if let Some(client) = self.client.clone() {
            if let Err(e) = client.publish(topic, qos, retain, payload).await {
                error!("Error during publish: {:?}", e);
            }
        }
    }
}
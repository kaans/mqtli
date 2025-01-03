use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::mqtli_config::{MqttBrokerConnect, MqttProtocol, TlsVersion};
use async_trait::async_trait;
use log::{debug, info};
use rumqttc::tokio_rustls::rustls::version::{TLS12, TLS13};
use rumqttc::tokio_rustls::rustls::{Certificate, PrivateKey, SupportedProtocolVersion};
use rumqttc::{TlsConfiguration, Transport};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;

pub mod v5;

pub mod mqtt_handler;
pub mod v311;

#[derive(Error, Debug)]
pub enum MqttServiceError {
    #[error("CA certificate must be present when using TLS")]
    CaCertificateMustBePresent(),
    #[error("Could not read CA certificate from file \"{1}\"")]
    CertificateNotReadable(#[source] io::Error, PathBuf),
    #[error("Could not add CA certificate to root store")]
    CaCertificateNotAdded(#[source] rumqttc::tokio_rustls::rustls::Error),
    #[error("Could not read client key from file \"{1}\"")]
    PrivateKeyNotReadable(#[source] io::Error, PathBuf),
    #[error("No PKCS8-encoded private key found in file \"{0}\"")]
    PrivateKeyNoneFound(PathBuf),
    #[error("More than one PKCS8-encoded private key found in file \"{0}\"")]
    PrivateKeyTooManyFound(PathBuf),
    #[error("Client key must be present when using TLS authentication")]
    ClientKeyMustBePresent(),
    #[error("Client error occurred")]
    ClientErrorV5(#[from] rumqttc::v5::ClientError),
    #[error("Client error occurred")]
    ClientErrorV311(#[from] rumqttc::ClientError),
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
pub enum QoS {
    #[default]
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl Display for QoS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            QoS::AtMostOnce => "At most once (0)",
            QoS::AtLeastOnce => "At least once (1)",
            QoS::ExactlyOnce => "Exactly once (2)",
        };
        write!(f, "{}", display)
    }
}

impl From<QoS> for rumqttc::v5::mqttbytes::QoS {
    fn from(value: QoS) -> Self {
        Self::from(&value)
    }
}

impl From<&QoS> for rumqttc::v5::mqttbytes::QoS {
    fn from(value: &QoS) -> Self {
        match value {
            QoS::AtMostOnce => rumqttc::v5::mqttbytes::QoS::AtMostOnce,
            QoS::AtLeastOnce => rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
            QoS::ExactlyOnce => rumqttc::v5::mqttbytes::QoS::ExactlyOnce,
        }
    }
}

impl From<QoS> for rumqttc::QoS {
    fn from(value: QoS) -> Self {
        Self::from(&value)
    }
}

impl From<&QoS> for rumqttc::QoS {
    fn from(value: &QoS) -> Self {
        match value {
            QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
            QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
            QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
        }
    }
}

impl From<rumqttc::QoS> for QoS {
    fn from(value: rumqttc::QoS) -> Self {
        Self::from(&value)
    }
}

impl From<&rumqttc::QoS> for QoS {
    fn from(value: &rumqttc::QoS) -> Self {
        match value {
            rumqttc::QoS::AtMostOnce => QoS::AtMostOnce,
            rumqttc::QoS::AtLeastOnce => QoS::AtLeastOnce,
            rumqttc::QoS::ExactlyOnce => QoS::ExactlyOnce,
        }
    }
}

impl From<rumqttc::v5::mqttbytes::QoS> for QoS {
    fn from(value: rumqttc::v5::mqttbytes::QoS) -> Self {
        Self::from(&value)
    }
}

impl From<&rumqttc::v5::mqttbytes::QoS> for QoS {
    fn from(value: &rumqttc::v5::mqttbytes::QoS) -> Self {
        match value {
            rumqttc::v5::mqttbytes::QoS::AtMostOnce => QoS::AtMostOnce,
            rumqttc::v5::mqttbytes::QoS::AtLeastOnce => QoS::AtLeastOnce,
            rumqttc::v5::mqttbytes::QoS::ExactlyOnce => QoS::ExactlyOnce,
        }
    }
}

#[async_trait]
pub trait MqttService: Send {
    async fn connect(
        &mut self,
        channel: Option<broadcast::Sender<MqttReceiveEvent>>,
        receiver_exit: Receiver<bool>,
    ) -> Result<JoinHandle<()>, MqttServiceError>;

    async fn disconnect(&self) -> Result<(), MqttServiceError>;

    async fn publish(&self, payload: MqttPublishEvent);

    async fn subscribe(&mut self, topic: String, qos: QoS);
}

#[derive(Clone)]
pub enum MqttReceiveEvent {
    V5(rumqttc::v5::Event),
    V311(rumqttc::Event),
}

#[derive(Clone, Debug)]
pub struct MqttPublishEvent {
    topic: String,
    qos: QoS,
    retain: bool,
    payload: Vec<u8>,
}

impl MqttPublishEvent {
    pub fn new(topic: String, qos: QoS, retain: bool, payload: Vec<u8>) -> Self {
        Self {
            topic,
            qos,
            retain,
            payload,
        }
    }
}

fn configure_tls_rustls(
    config: Arc<MqttBrokerConnect>,
) -> Result<TlsConfiguration, MqttServiceError> {
    fn load_private_key_from_file(path: &PathBuf) -> Result<PrivateKey, MqttServiceError> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                return Err(MqttServiceError::PrivateKeyNotReadable(
                    e,
                    PathBuf::from(path),
                ));
            }
        };
        let mut reader = BufReader::new(file);
        let mut keys = match rustls_pemfile::pkcs8_private_keys(&mut reader) {
            Ok(keys) => keys,
            Err(e) => {
                return Err(MqttServiceError::PrivateKeyNotReadable(
                    e,
                    PathBuf::from(path),
                ));
            }
        };

        match keys.len() {
            0 => Err(MqttServiceError::PrivateKeyNoneFound(PathBuf::from(path))),
            1 => Ok(PrivateKey(keys.remove(0))),
            _ => Err(MqttServiceError::PrivateKeyTooManyFound(PathBuf::from(
                path,
            ))),
        }
    }

    fn load_certificates_from_file(path: &PathBuf) -> Result<Vec<Certificate>, MqttServiceError> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                return Err(MqttServiceError::CertificateNotReadable(
                    e,
                    PathBuf::from(path),
                ));
            }
        };
        let mut reader = BufReader::new(file);
        let certs = match rustls_pemfile::certs(&mut reader) {
            Ok(certs) => certs,
            Err(e) => {
                return Err(MqttServiceError::CertificateNotReadable(
                    e,
                    PathBuf::from(path),
                ));
            }
        };

        Ok(certs.into_iter().map(Certificate).collect())
    }

    let mut root_store = rumqttc::tokio_rustls::rustls::RootCertStore::empty();

    match &config.tls_ca_file() {
        Some(ca_file) => {
            let certificates = load_certificates_from_file(ca_file)?;

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

    let tls_config = rumqttc::tokio_rustls::rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups();

    let pr: Vec<&'static SupportedProtocolVersion> = match config.tls_version() {
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

    let tls_config = tls_config
        .with_protocol_versions(pr.as_slice())
        .unwrap()
        .with_root_certificates(root_store);

    let tls_config = match config.tls_client_certificate() {
        None => tls_config.with_no_client_auth(),
        Some(client_certificate_file) => {
            info!("Using TLS client certificate authentication");

            let client_certificate = load_certificates_from_file(client_certificate_file)?;

            let Some(client_key_file) = config.tls_client_key() else {
                return Err(MqttServiceError::ClientKeyMustBePresent());
            };

            let client_key = load_private_key_from_file(client_key_file)?;

            tls_config
                .with_client_auth_cert(client_certificate, client_key)
                .unwrap()
        }
    };

    Ok(TlsConfiguration::Rustls(Arc::new(tls_config)))
}

fn get_transport_parameters(
    config: Arc<MqttBrokerConnect>,
) -> Result<(Transport, String), MqttServiceError> {
    let (transport, hostname) = match config.protocol() {
        MqttProtocol::Tcp => match *config.use_tls() {
            false => {
                debug!("Using TCP");
                (Transport::Tcp, config.host().to_string())
            }
            true => {
                debug!("Using TCP with TLS");
                (
                    Transport::Tls(configure_tls_rustls(config.clone())?),
                    config.host().to_string(),
                )
            }
        },
        MqttProtocol::Websocket => match *config.use_tls() {
            false => {
                debug!("Using websockets");

                let hostname = format!("ws://{}:{}/mqtt", config.host(), config.port());
                (Transport::Ws, hostname)
            }
            true => {
                debug!("Using websockets with TLS");

                let hostname = format!("wss://{}:{}/mqtt", config.host(), config.port());
                (
                    Transport::Wss(configure_tls_rustls(config.clone())?),
                    hostname,
                )
            }
        },
    };
    Ok((transport, hostname))
}

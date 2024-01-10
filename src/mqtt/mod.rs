use std::io;
use std::path::PathBuf;

use async_trait::async_trait;
use rumqttc::v5::Event;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

pub mod v5;

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
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum QoS {
    #[default]
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
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

#[async_trait]
pub trait MqttService: Send {
    async fn connect(
        &mut self,
        channel: Option<broadcast::Sender<Event>>,
    ) -> Result<JoinHandle<()>, MqttServiceError>;

    async fn disconnect(&self);

    async fn subscribe(&mut self, topic: String, qos: QoS);

    async fn publish(&self, topic: String, qos: QoS, retain: bool, payload: Vec<u8>);
}
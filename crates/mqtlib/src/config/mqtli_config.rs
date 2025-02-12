use crate::config::sql_storage::SqlStorage;
use crate::config::topic::TopicStorage;
use crate::mqtt::QoS;
use derive_builder::Builder;
use derive_getters::Getters;
use serde::Deserialize;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::time::Duration;
use tracing::Level;
use validator::{Validate, ValidationError};

#[derive(Debug, Getters, Validate, Builder)]
pub struct MqtliConfig {
    #[validate(nested)]
    pub broker: MqttBrokerConnect,
    pub log_level: Level,
    #[validate(nested)]
    pub topic_storage: TopicStorage,
    pub mode: Mode,
    #[validate(nested)]
    pub sql_storage: Option<SqlStorage>,
}

impl Display for MqtliConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parsed configuration:")?;
        write!(f, "topics:")?;
        for topic in &self.topic_storage.topics {
            write!(f, "\n\n{}", topic)?;
        }

        Ok(())
    }
}

impl Default for MqtliConfig {
    fn default() -> Self {
        Self {
            broker: Default::default(),
            log_level: Level::INFO,
            topic_storage: TopicStorage::default(),
            mode: Default::default(),
            sql_storage: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub enum Mode {
    #[default]
    MultiTopic,
    Publish,
    Subscribe,
    Sparkplug,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::MultiTopic => write!(f, "Multi-Topic"),
            Mode::Publish => write!(f, "Publish"),
            Mode::Subscribe => write!(f, "Subscribe"),
            Mode::Sparkplug => write!(f, "Sparkplug"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub enum TlsVersion {
    #[default]
    #[serde(rename = "all")]
    All,
    #[serde(rename = "v12")]
    Version1_2,
    #[serde(rename = "v13")]
    Version1_3,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub enum MqttVersion {
    #[serde(rename = "v311")]
    V311,

    #[default]
    #[serde(rename = "v5")]
    V5,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub enum MqttProtocol {
    #[default]
    #[serde(rename = "tcp")]
    Tcp,

    #[serde(rename = "websocket")]
    Websocket,
}

#[derive(Clone, Debug, Getters, Validate, Builder)]
#[validate(schema(function = "validate_credentials", skip_on_field_errors = false))]
#[validate(schema(function = "validate_tls_client"))]
pub struct MqttBrokerConnect {
    #[validate(length(min = 1, message = "Hostname must be given"))]
    pub host: String,
    pub port: u16,
    pub protocol: MqttProtocol,

    #[validate(length(min = 1, message = "Client id must be given"))]
    pub client_id: String,
    pub mqtt_version: MqttVersion,
    #[validate(custom(
        function = "validate_keep_alive",
        message = "Keep alive must be a number and at least 5 seconds"
    ))]
    pub keep_alive: Duration,
    pub username: Option<String>,
    pub password: Option<String>,

    pub use_tls: bool,
    pub tls_ca_file: Option<PathBuf>,
    pub tls_client_certificate: Option<PathBuf>,
    pub tls_client_key: Option<PathBuf>,
    pub tls_version: TlsVersion,

    #[validate(nested)]
    pub last_will: Option<LastWillConfig>,
}

impl Default for MqttBrokerConnect {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 1883,
            protocol: MqttProtocol::Tcp,
            client_id: "mqtli".to_string(),
            mqtt_version: MqttVersion::V5,
            keep_alive: Duration::from_secs(5),
            username: None,
            password: None,
            use_tls: false,
            tls_ca_file: None,
            tls_client_certificate: None,
            tls_client_key: None,
            tls_version: Default::default(),
            last_will: None,
        }
    }
}

#[derive(Clone, Debug, Default, Getters, Validate, Builder)]
pub struct LastWillConfig {
    #[validate(length(min = 1, message = "Last will topic must be given"))]
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: QoS,
    pub retain: bool,
}

fn validate_keep_alive(value: &Duration) -> Result<(), ValidationError> {
    if value.as_secs() >= 5 {
        return Ok(());
    }

    let mut err = ValidationError::new("wrong_keep_alive");
    err.message = Some(Cow::from("Keep alive must be at least 5 seconds"));

    Err(err)
}

fn validate_credentials(value: &MqttBrokerConnect) -> Result<(), ValidationError> {
    let mut err = ValidationError::new("wrong_credentials");

    if value.username.is_none() && value.password.is_some() {
        err.message = Some(Cow::from("Password is given but no username"));
        return Err(err);
    } else if value.username.is_some() && value.password.is_none() {
        err.message = Some(Cow::from("Username is given but no password"));
        return Err(err);
    }

    Ok(())
}

fn validate_tls_client(value: &MqttBrokerConnect) -> Result<(), ValidationError> {
    let mut err = ValidationError::new("wrong_tls_client");

    if value.tls_client_key.is_none() && value.tls_client_certificate.is_some() {
        err.message = Some(Cow::from("TLS client certificate is given but no key"));
        return Err(err);
    } else if value.tls_client_key.is_some() && value.tls_client_certificate.is_none() {
        err.message = Some(Cow::from("TLS client key is given but no certificate"));
        return Err(err);
    }

    Ok(())
}

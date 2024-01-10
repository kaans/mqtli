use std::borrow::Cow;
use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;

use clap::ValueEnum;
use derive_getters::Getters;
use log::LevelFilter;
use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::config::args;
use crate::config::args::{read_cli_args, read_config};
use crate::config::{ConfigError, PayloadType, PublishInputType};
use crate::mqtt::QoS;

#[derive(Debug, Getters, Validate)]
pub struct MqtliConfig {
    #[validate]
    broker: MqttBrokerConnectArgs,
    log_level: LevelFilter,
    #[validate]
    pub topics: Vec<Topic>,
}

impl MqtliConfig {
    fn merge(&mut self, other: &args::MqtliArgs) {
        if let Some(broker) = &other.broker {
            self.broker.merge(broker);
        }

        if let Some(log_level) = other.log_level {
            self.log_level = log_level
        };
        other
            .topics
            .iter()
            .for_each(|topic| self.topics.push(Topic::from(topic)));
    }
}

impl Default for MqtliConfig {
    fn default() -> Self {
        Self {
            broker: Default::default(),
            log_level: LevelFilter::Info,
            topics: vec![],
        }
    }
}

#[derive(Debug, Default, Getters, Validate)]
pub struct Topic {
    #[validate(length(min = 1, message = "Topic must be given"))]
    topic: String,
    #[validate]
    subscription: Subscription,
    payload: PayloadType,
    #[validate]
    publish: Option<Publish>,
}

#[derive(Debug, Getters, Validate)]
pub struct Publish {
    enabled: bool,
    qos: QoS,
    retain: bool,
    trigger: Vec<PublishTriggerType>,
    #[validate]
    input: PublishInputType,
}

impl Default for Publish {
    fn default() -> Self {
        Publish {
            enabled: true,
            qos: Default::default(),
            retain: false,
            trigger: vec![],
            input: Default::default(),
        }
    }
}

impl From<&args::Publish> for Publish {
    fn from(value: &args::Publish) -> Self {
        let trigger: Vec<PublishTriggerType> = match value.trigger() {
            None => {
                vec![PublishTriggerType::default()]
            }
            Some(trigger) => trigger.iter().map(PublishTriggerType::from).collect(),
        };

        Publish {
            enabled: *value.enabled(),
            qos: *value.qos(),
            retain: *value.retain(),
            trigger,
            input: (*value.input()).clone(),
        }
    }
}

#[derive(Debug, Getters, Validate)]
pub struct PublishTriggerTypePeriodic {
    interval: Duration,
    count: Option<u32>,
    initial_delay: Duration,
}

impl Default for PublishTriggerTypePeriodic {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
            count: None,
            initial_delay: Duration::from_secs(0),
        }
    }
}

impl From<&args::PublishTriggerTypePeriodic> for PublishTriggerTypePeriodic {
    fn from(value: &args::PublishTriggerTypePeriodic) -> Self {
        let default = Self::default();

        Self {
            interval: match value.interval() {
                None => default.interval,
                Some(value) => *value,
            },
            count: match value.count() {
                None => default.count,
                Some(value) => Some(*value),
            },
            initial_delay: match value.initial_delay() {
                None => default.initial_delay,
                Some(value) => *value,
            },
        }
    }
}

#[derive(Debug)]
pub enum PublishTriggerType {
    Periodic(PublishTriggerTypePeriodic),
}

impl From<&args::PublishTriggerType> for PublishTriggerType {
    fn from(value: &args::PublishTriggerType) -> Self {
        match value {
            args::PublishTriggerType::Periodic(value) => {
                PublishTriggerType::Periodic(PublishTriggerTypePeriodic::from(value))
            }
        }
    }
}

impl Default for PublishTriggerType {
    fn default() -> Self {
        Self::Periodic(PublishTriggerTypePeriodic::default())
    }
}

#[derive(Debug, Default, Getters, Validate)]
pub struct Output {
    format: PayloadType,
    target: OutputTarget,
}

impl From<&args::Output> for Output {
    fn from(value: &args::Output) -> Self {
        Output {
            format: match value.format() {
                None => PayloadType::default(),
                Some(value) => value.clone(),
            },
            target: match value.target() {
                None => OutputTarget::Console(OutputTargetConsole::default()),
                Some(value) => match value {
                    args::OutputTarget::Console(options) => {
                        OutputTarget::Console(OutputTargetConsole::from(options))
                    }
                    args::OutputTarget::File(options) => {
                        OutputTarget::File(OutputTargetFile::from(options))
                    }
                },
            },
        }
    }
}

#[derive(Debug)]
pub enum OutputTarget {
    Console(OutputTargetConsole),
    File(OutputTargetFile),
}

impl Default for OutputTarget {
    fn default() -> Self {
        OutputTarget::Console(OutputTargetConsole::default())
    }
}

#[derive(Debug, Default, Getters, Validate)]
pub struct OutputTargetConsole {}

impl From<&args::OutputTargetConsole> for OutputTargetConsole {
    fn from(_: &args::OutputTargetConsole) -> Self {
        Self {}
    }
}

#[derive(Debug, Getters, Validate)]
pub struct OutputTargetFile {
    path: PathBuf,
    overwrite: bool,
    prepend: Option<String>,
    append: Option<String>,
}

impl From<&args::OutputTargetFile> for OutputTargetFile {
    fn from(value: &args::OutputTargetFile) -> Self {
        Self {
            path: PathBuf::from(value.path()),
            overwrite: *value.overwrite(),
            prepend: value.prepend().clone(),
            append: value.append().clone(),
        }
    }
}

impl Default for OutputTargetFile {
    fn default() -> Self {
        OutputTargetFile {
            path: Default::default(),
            overwrite: false,
            prepend: None,
            append: Some("\n".to_string()),
        }
    }
}

#[derive(Debug, Getters, Validate)]
pub struct Subscription {
    enabled: bool,
    qos: QoS,
    outputs: Vec<Output>,
}

impl Default for Subscription {
    fn default() -> Self {
        Subscription {
            enabled: true,
            qos: Default::default(),
            outputs: vec![],
        }
    }
}

impl From<&args::Subscription> for Subscription {
    fn from(value: &args::Subscription) -> Self {
        let outputs: Vec<Output> = match value.outputs() {
            None => {
                vec![Output::default()]
            }
            Some(outputs) => outputs.iter().map(Output::from).collect(),
        };

        Subscription {
            enabled: *value.enabled(),
            qos: *value.qos(),
            outputs,
        }
    }
}

impl From<&args::Topic> for Topic {
    fn from(value: &args::Topic) -> Self {
        Topic {
            topic: String::from(value.topic()),
            subscription: match value.subscription() {
                None => Subscription::default(),
                Some(value) => Subscription::from(value),
            },
            payload: match value.payload() {
                None => PayloadType::default(),
                Some(value) => value.clone(),
            },
            publish: value.publish().as_ref().map(Publish::from),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
pub enum TlsVersion {
    #[default]
    #[serde(rename = "all")]
    #[clap(name = "all")]
    All,
    #[serde(rename = "v12")]
    #[clap(name = "v12")]
    Version1_2,
    #[serde(rename = "v13")]
    #[clap(name = "v13")]
    Version1_3,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
pub enum MqttVersion {
    #[serde(rename = "v311")]
    #[clap(name = "v311")]
    V311,

    #[default]
    #[serde(rename = "v5")]
    #[clap(name = "v5")]
    V5,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
pub enum MqttProtocol {
    #[default]
    #[serde(rename = "tcp")]
    #[clap(name = "tcp")]
    Tcp,

    #[serde(rename = "websocket")]
    #[clap(name = "websocket")]
    Websocket,
}

#[derive(Clone, Debug, Getters, Validate)]
#[validate(schema(function = "validate_credentials", skip_on_field_errors = false))]
#[validate(schema(function = "validate_tls_client", skip_on_field_errors = false))]
pub struct MqttBrokerConnectArgs {
    #[validate(length(min = 1, message = "Hostname must be given"))]
    host: String,
    port: u16,
    protocol: MqttProtocol,

    #[validate(length(min = 1, message = "Client id must be given"))]
    client_id: String,
    mqtt_version: MqttVersion,
    #[validate(custom(
        function = "validate_keep_alive",
        message = "Keep alive must be a number and at least 5 seconds"
    ))]
    keep_alive: Duration,
    username: Option<String>,
    password: Option<String>,

    use_tls: bool,
    tls_ca_file: Option<PathBuf>,
    tls_client_certificate: Option<PathBuf>,
    tls_client_key: Option<PathBuf>,
    tls_version: TlsVersion,

    #[validate]
    last_will: Option<LastWillConfig>,
}

impl MqttBrokerConnectArgs {
    fn merge(&mut self, other: &args::MqttBrokerConnectArgs) {
        if let Some(host) = &other.host {
            self.host = host.to_string()
        }
        if let Some(port) = other.port {
            self.port = port
        }
        if let Some(protocol) = &other.protocol {
            self.protocol = protocol.clone()
        }
        if let Some(client_id) = &other.client_id {
            self.client_id = client_id.to_string()
        }
        if let Some(mqtt_version) = &other.mqtt_version {
            self.mqtt_version = mqtt_version.clone()
        }
        if let Some(keep_alive) = other.keep_alive {
            self.keep_alive = keep_alive
        }
        if let Some(username) = &other.username {
            self.username = Some(username.to_string())
        }
        if let Some(password) = &other.password {
            self.password = Some(password.to_string())
        }
        if let Some(use_tls) = other.use_tls {
            self.use_tls = use_tls
        }
        if let Some(tls_ca_file) = &other.tls_ca_file {
            self.tls_ca_file = Some(PathBuf::from(tls_ca_file))
        }
        if let Some(tls_client_certificate) = &other.tls_client_certificate {
            self.tls_client_certificate = Some(PathBuf::from(tls_client_certificate))
        }
        if let Some(tls_client_key) = &other.tls_client_key {
            self.tls_client_key = Some(PathBuf::from(tls_client_key))
        }
        if let Some(tls_version) = &other.tls_version {
            self.tls_version = tls_version.clone()
        }

        if let Some(last_will) = &other.last_will {
            let mut lw = self.last_will.clone().unwrap_or_default();

            if let Some(topic) = &last_will.topic {
                lw.topic = topic.to_string()
            };
            if let Some(qos) = &last_will.qos {
                lw.qos = *qos
            };
            if let Some(payload) = &last_will.payload {
                lw.payload = payload.clone().into_bytes()
            };
            if let Some(retain) = &last_will.retain {
                lw.retain = *retain
            };

            self.last_will = Some(lw);
        }
    }
}

impl Default for MqttBrokerConnectArgs {
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

#[derive(Clone, Debug, Default, Getters, Validate)]
pub struct LastWillConfig {
    #[validate(length(min = 1, message = "Last will topic must be given"))]
    topic: String,
    payload: Vec<u8>,
    qos: QoS,
    retain: bool,
}

pub fn parse_config() -> Result<MqtliConfig, ConfigError> {
    let args = read_cli_args();
    let config_file = match &args.config_file {
        None => PathBuf::from("config.yaml"),
        Some(config_file) => config_file.to_path_buf(),
    };

    let config_file = read_config(&config_file)?;

    let mut config = MqtliConfig {
        ..Default::default()
    };

    config.merge(&config_file);
    config.merge(&args);

    match config.validate() {
        Ok(_) => Ok(config),
        Err(e) => Err(ConfigError::InvalidConfiguration(e)),
    }
}

fn validate_keep_alive(value: &Duration) -> Result<(), ValidationError> {
    if value.as_secs() >= 5 {
        return Ok(());
    }

    let mut err = ValidationError::new("wrong_keep_alive");
    err.message = Some(Cow::from("Keep alive must be at least 5 seconds"));

    Err(err)
}

fn validate_credentials(value: &MqttBrokerConnectArgs) -> Result<(), ValidationError> {
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

fn validate_tls_client(value: &MqttBrokerConnectArgs) -> Result<(), ValidationError> {
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

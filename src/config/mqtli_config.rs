use std::borrow::Cow;
use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use derive_getters::Getters;
use log::LevelFilter;
use rumqttc::v5::mqttbytes::QoS;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::config::args::{LastWillConfig as ArgsLastWillConfig,
                          MqtliArgs};
use crate::config::config_file::{LastWillConfig as ConfigFileLastWillConfig, Output as ConfigFileOutput, OutputFormat as ConfigFileOutputFormat, OutputTarget as ConfigFileOutputTarget, OutputTargetConsole as ConfigFileOutputTargetConsole, OutputTargetFile as ConfigFileOutputTargetFile, PayloadProtobuf as ConfigFilePayloadProtobuf, PayloadText as ConfigFilePayloadText, PayloadType as ConfigFilePayloadType, read_config, Subscription as ConfigFileSubscription, Topic as ConfigFileTopic};
use crate::config::ConfigError;
use crate::config::mqtli_config::PayloadType::Text;

#[derive(Clone, Debug, Default, Getters, Validate)]
pub struct MqtliConfig {
    #[validate]
    pub broker: MqttBrokerConnectArgs,

    logger: LoggingArgs,

    _config_file: PathBuf,

    topics: Vec<Topic>,
}

#[derive(Clone, Debug, Default, Getters, Validate)]
pub struct Topic {
    #[validate(length(min = 1, message = "Topic must be given"))]
    topic: String,
    subscription: Subscription,
    payload: PayloadType,
    outputs: Vec<Output>,
}

#[derive(Clone, Debug, Default, Getters, Validate)]
pub struct Output {
    format: OutputFormat,
    target: OutputTarget,
}

impl From<&ConfigFileOutput> for Output {
    fn from(value: &ConfigFileOutput) -> Self {
        Output {
            format: match value.format() {
                None => OutputFormat::Text,
                Some(value) => {
                    match value {
                        ConfigFileOutputFormat::Text => OutputFormat::Text,
                        ConfigFileOutputFormat::Json => OutputFormat::Json,
                        ConfigFileOutputFormat::Yaml => OutputFormat::Yaml,
                        ConfigFileOutputFormat::Hex => OutputFormat::Hex,
                        ConfigFileOutputFormat::Base64 => OutputFormat::Base64,
                        ConfigFileOutputFormat::Raw => OutputFormat::Raw,
                    }
                }
            },
            target: match value.target() {
                None => OutputTarget::Console(OutputTargetConsole::default()),
                Some(value) => {
                    match value {
                        ConfigFileOutputTarget::Console(options)
                        => OutputTarget::Console(OutputTargetConsole::from(options)),
                        ConfigFileOutputTarget::File(options)
                        => OutputTarget::File(OutputTargetFile::from(options))
                    }
                }
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Hex,
    Base64,
    Raw,
}

impl Default for OutputFormat {
    fn default() -> Self { OutputFormat::Text }
}

#[derive(Clone, Debug)]
pub enum OutputTarget {
    Console(OutputTargetConsole),
    File(OutputTargetFile),
}

impl Default for OutputTarget {
    fn default() -> Self { OutputTarget::Console(OutputTargetConsole::default()) }
}

#[derive(Clone, Debug, Default, Getters, Validate)]
pub struct OutputTargetConsole {}

impl From<&ConfigFileOutputTargetConsole> for OutputTargetConsole {
    fn from(_value: &ConfigFileOutputTargetConsole) -> Self {
        OutputTargetConsole {}
    }
}

#[derive(Clone, Debug, Getters, Validate)]
pub struct OutputTargetFile {
    path: PathBuf,
    overwrite: bool,
    prepend: Option<String>,
    append: Option<String>,
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

impl From<&ConfigFileOutputTargetFile> for OutputTargetFile {
    fn from(value: &ConfigFileOutputTargetFile) -> Self {
        OutputTargetFile {
            path: PathBuf::from(value.path()),
            overwrite: *value.overwrite(),
            prepend: value.prepend().clone(),
            append: value.append().clone().or(OutputTargetFile::default().append),
        }
    }
}

#[derive(Clone, Debug, Default, Getters, Validate)]
pub struct Subscription {
    enabled: bool,
    qos: QoS,
}

impl From<&ConfigFileSubscription> for Subscription {
    fn from(value: &ConfigFileSubscription) -> Self {
        Subscription {
            enabled: *value.enabled(),
            qos: *value.qos(),
        }
    }
}

impl From<&ConfigFileTopic> for Topic {
    fn from(value: &ConfigFileTopic) -> Self {
        let outputs: Vec<Output> =
            match value.outputs() {
                None => {
                    vec![Output::default()]
                }
                Some(outputs) => {
                    outputs.iter().map(|output| {
                        Output::from(output)
                    }).collect()
                }
            };

        Topic {
            topic: String::from(value.topic()),
            subscription: match value.subscription() {
                None => { Subscription::default() }
                Some(value) => {
                    Subscription::from(value)
                }
            },
            payload: match value.payload() {
                None => PayloadType::default(),
                Some(value) => {
                    PayloadType::from(value)
                }
            },
            outputs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PayloadType {
    Text(PayloadText),
    Protobuf(PayloadProtobuf),
}

impl Default for PayloadType {
    fn default() -> Self {
        Text(PayloadText::default())
    }
}

impl From<&ConfigFilePayloadType> for PayloadType {
    fn from(value: &ConfigFilePayloadType) -> Self {
        match value {
            ConfigFilePayloadType::Text(v) => PayloadType::Text(PayloadText::from(v)),
            ConfigFilePayloadType::Protobuf(v) => PayloadType::Protobuf(PayloadProtobuf::from(v))
        }
    }
}

#[derive(Clone, Debug, Default, Getters, PartialEq)]
pub struct PayloadText {}

impl From<&ConfigFilePayloadText> for PayloadText {
    fn from(_value: &ConfigFilePayloadText) -> Self {
        PayloadText {}
    }
}

#[derive(Clone, Debug, Default, Getters, PartialEq, Validate)]
pub struct PayloadProtobuf {
    definition: PathBuf,
    #[validate(length(min = 1, message = "Message must be given"))]
    message: String,
}

impl From<&ConfigFilePayloadProtobuf> for PayloadProtobuf {
    fn from(value: &ConfigFilePayloadProtobuf) -> Self {
        PayloadProtobuf {
            definition: PathBuf::from(value.definition()),
            message: String::from(value.message()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ValueEnum)]
pub enum TlsVersion {
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

impl Default for TlsVersion {
    fn default() -> Self {
        TlsVersion::All
    }
}

#[derive(Clone, Debug, Default, Getters, Validate)]
#[validate(schema(function = "validate_credentials", skip_on_field_errors = false))]
#[validate(schema(function = "validate_tls_client", skip_on_field_errors = false))]
pub struct MqttBrokerConnectArgs {
    #[validate(length(min = 1, message = "Hostname must be given"))]
    host: String,
    port: u16,
    #[validate(length(min = 1, message = "Client id must be given"))]
    client_id: String,
    #[validate(custom(function = "validate_keep_alive", message = "Keep alive must be a number and at least 5 seconds"))]
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

#[derive(Clone, Debug, Default, Getters, Validate)]
pub struct LastWillConfig {
    #[validate(length(min = 1, message = "Last will topic must be given"))]
    topic: String,
    payload: Vec<u8>,
    qos: QoS,
    retain: bool,
}

impl From<&ConfigFileLastWillConfig> for LastWillConfig {
    fn from(value: &ConfigFileLastWillConfig) -> Self {
        Self {
            topic: match value.topic() {
                None => { String::default() }
                Some(value) => {
                    String::from(value)
                }
            },
            payload: match value.payload() {
                None => vec![],
                Some(payload) => Vec::from(payload.clone())
            },
            qos: match value.qos() {
                None => QoS::AtMostOnce,
                Some(qos) => *qos
            },
            retain: match value.retain() {
                None => false,
                Some(retain) => *retain
            },
        }
    }
}

impl From<&ArgsLastWillConfig> for LastWillConfig {
    fn from(value: &ArgsLastWillConfig) -> Self {
        Self {
            topic: match value.topic() {
                None => { String::default() }
                Some(value) => {
                    String::from(value)
                }
            },
            payload: match value.payload() {
                None => vec![],
                Some(payload) => Vec::from(payload.clone())
            },
            qos: match value.qos() {
                None => QoS::AtMostOnce,
                Some(qos) => *qos
            },
            retain: match value.retain() {
                None => false,
                Some(retain) => *retain
            },
        }
    }
}

#[derive(Clone, Debug, Getters)]
pub struct LoggingArgs {
    level: LevelFilter,
}

impl Default for LoggingArgs {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
        }
    }
}

pub fn parse_config() -> Result<MqtliConfig, ConfigError> {
    let args = MqtliArgs::parse();
    let config_file = read_config(&args.config_file())?;

    let mut config = MqtliConfig {
        ..Default::default()
    };

    config.broker.host = args.broker().host().clone().or(config_file.host().clone()).or(Some("localhost".to_string())).unwrap();
    config.broker.port = args.broker().port().or(config_file.port().clone()).or(Some(1883)).unwrap();
    config.broker.client_id = args.broker().client_id().clone().or(config_file.client_id().clone()).or(Some("mqtli".to_string())).unwrap();
    config.broker.keep_alive = args.broker().keep_alive().or(config_file.keep_alive().clone()).or(Some(Duration::from_secs(5))).unwrap();
    config.broker.username = args.broker().username().clone().or(config_file.username().clone()).or(None);
    config.broker.password = args.broker().password().clone().or(config_file.password().clone()).or(None);

    config.broker.use_tls = args.broker().use_tls().clone().or(config_file.use_tls().clone()).or(Some(false)).unwrap();
    config.broker.tls_ca_file = args.broker().tls_ca_file().clone().or(config_file.tls_ca_file().clone()).or(None);
    config.broker.tls_client_certificate = args.broker().tls_client_certificate().clone().or(config_file.tls_client_certificate().clone()).or(None);
    config.broker.tls_client_key = args.broker().tls_client_key().clone().or(config_file.tls_client_key().clone()).or(None);
    config.broker.tls_version = args.broker().tls_version().clone().or(config_file.tls_version().clone()).or(Some(TlsVersion::All)).unwrap();

    {
        let last_will = if args.broker().last_will().is_none() && config_file.last_will().is_none() {
            None
        } else {
            let mut lwc = LastWillConfig {
                topic: "".to_string(),
                payload: vec![],
                qos: Default::default(),
                retain: false,
            };

            if let Some(lw_args) = config_file.last_will() {
                if lw_args.topic().is_some() {
                    lwc.topic = lw_args.topic().clone().unwrap();
                }
                if lw_args.payload().is_some() {
                    lwc.payload = lw_args.payload().clone().unwrap().into_bytes();
                }
                if lw_args.qos().is_some() {
                    lwc.qos = lw_args.qos().clone().unwrap();
                }
                if lw_args.retain().is_some() {
                    lwc.retain = lw_args.retain().clone().unwrap();
                }
            }

            if let Some(lw_args) = args.broker().last_will() {
                if lw_args.topic().is_some() {
                    lwc.topic = lw_args.topic().clone().unwrap();
                }
                if lw_args.payload().is_some() {
                    lwc.payload = lw_args.payload().clone().unwrap().into_bytes();
                }
                if lw_args.qos().is_some() {
                    lwc.qos = lw_args.qos().clone().unwrap();
                }
                if lw_args.retain().is_some() {
                    lwc.retain = lw_args.retain().clone().unwrap();
                }
            }

            Some(lwc)
        };

        config.broker.last_will = last_will;
    }

    config.logger.level = args.logger().level().or(config_file.log_level().clone()
        .map(|v| LevelFilter::from_str(v.as_str()).expect("Invalid log level {v}")))
        .or(Option::from(LevelFilter::Info)).unwrap();

    for topic in config_file.topics() {
        config.topics.push(Topic::from(topic));
    }

    return match config.validate() {
        Ok(_) => Ok(config),
        Err(e) => Err(ConfigError::InvalidConfiguration(e))
    };
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

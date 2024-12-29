use crate::args::parsers::deserialize_duration_seconds;
use crate::args::parsers::deserialize_level_filter;
use crate::args::parsers::deserialize_qos_option;
use crate::args::parsers::parse_keep_alive;
use crate::args::parsers::parse_qos;
use crate::args::ArgsError;
use clap::{Args, Parser, ValueEnum};
use derive_getters::Getters;
use log::LevelFilter;
use mqtlib::config::mqtli_config::{
    LastWillConfig, LastWillConfigBuilder, MqtliConfig, MqtliConfigBuilder, MqttBrokerConnect,
    MqttBrokerConnectBuilder,
};
use mqtlib::config::topic::Topic;
use mqtlib::mqtt::QoS;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Deserialize, Parser)]
#[command(author, version, about, long_about = None)]
#[clap(disable_version_flag = true)]
#[clap(disable_help_flag = true)]
pub struct MqtliArgs {
    #[clap(long, action = clap::ArgAction::HelpLong, help = "Print help")]
    help: Option<bool>,

    #[clap(long, action = clap::ArgAction::Version, help = "Print version")]
    version: Option<bool>,

    #[command(flatten)]
    pub broker: Option<MqttBrokerConnectArgs>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_level_filter")]
    #[arg(
        short = 'l',
        long = "log-level",
        env = "LOG_LEVEL",
        help_heading = "Logging",
        help = "Log level (default: info) (possible values: trace, debug, info, warn, error, off)"
    )]
    pub log_level: Option<LevelFilter>,

    #[arg(
        short = 'c',
        long = "config-file",
        env = "CONFIG_FILE_PATH",
        help = "Path to the config file (default: config.yaml)"
    )]
    #[serde(skip_serializing)]
    pub config_file: Option<PathBuf>,

    #[clap(skip)]
    #[serde(default)]
    pub topics: Vec<Topic>,
}

impl MqtliArgs {
    pub fn merge(self, other: MqtliConfig) -> Result<MqtliConfig, ArgsError> {
        let mut builder = MqtliConfigBuilder::default();

        builder.broker(match self.broker {
            None => other.broker,
            Some(broker) => broker.merge(other.broker)?,
        });

        builder.log_level(match self.log_level {
            None => other.log_level,
            Some(log_level) => log_level,
        });

        builder.topics(other.topics.into_iter().chain(self.topics).collect());

        builder.build().map_err(ArgsError::from)
    }
}

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct MqttBrokerConnectArgs {
    #[arg(
        short = 'h',
        long = "host",
        env = "BROKER_HOST",
        help_heading = "Broker",
        help = "The ip address or hostname of the broker (default: localhost)"
    )]
    pub host: Option<String>,

    #[arg(
        short = 'p',
        long = "port",
        env = "BROKER_PORT",
        help_heading = "Broker",
        help = "The port the broker is listening on (default: 1883)"
    )]
    pub port: Option<u16>,

    #[arg(
        long = "protocol",
        env = "BROKER_PROTOCOL",
        help_heading = "Broker",
        help = "The protocol to use to communicate with the broker (default: tcp)"
    )]
    pub protocol: Option<MqttProtocol>,

    #[arg(
        short = 'i',
        long = "client-id",
        env = "BROKER_CLIENT_ID",
        help_heading = "Broker",
        help = "The client id for this mqtli instance (default: mqtli)"
    )]
    pub client_id: Option<String>,

    #[arg(
        short = 'v',
        long = "mqtt-version",
        env = "BROKER_MQTT_VERSION",
        help_heading = "Broker",
        help = "The MQTT version to use (default: v5)"
    )]
    pub mqtt_version: Option<MqttVersion>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    #[arg(long = "keep-alive", env = "BROKER_KEEP_ALIVE", value_parser = parse_keep_alive, help_heading = "Broker", help = "Keep alive time in seconds (default: 5 seconds)"
    )]
    pub keep_alive: Option<Duration>,

    #[arg(
        short = 'u',
        long = "username",
        env = "BROKER_USERNAME",
        help_heading = "Broker",
        help = "(optional) Username used to authenticate against the broker; if used then username must be given too (default: empty)"
    )]
    pub username: Option<String>,

    #[arg(
        short = 'w',
        long = "password",
        env = "BROKER_PASSWORD",
        help_heading = "Broker",
        help = "(optional) Password used to authenticate against the broker; if used then password must be given too (default: empty)"
    )]
    pub password: Option<String>,

    #[arg(
        long = "use-tls",
        env = "BROKER_USE_TLS",
        help_heading = "TLS",
        help = "If specified, TLS is used to communicate with the broker (default: false)"
    )]
    pub use_tls: Option<bool>,

    #[arg(
        long = "ca-file",
        env = "BROKER_TLS_CA_FILE",
        help_heading = "TLS",
        help = "Path to a PEM encoded ca certificate to verify the broker's certificate (default: empty)"
    )]
    pub tls_ca_file: Option<PathBuf>,

    #[arg(
        long = "client-cert",
        env = "BROKER_TLS_CLIENT_CERTIFICATE_FILE",
        help_heading = "TLS",
        help = "(optional) Path to a PEM encoded client certificate for authenticating against the broker; must be specified with client-key (default: empty)"
    )]
    pub tls_client_certificate: Option<PathBuf>,

    #[arg(
        long = "client-key",
        env = "BROKER_TLS_CLIENT_KEY_FILE",
        help_heading = "TLS",
        help = "(optional) Path to a PKCS#8 encoded, unencrypted client private key for authenticating against the broker; must be specified with client-cert (default: empty)"
    )]
    pub tls_client_key: Option<PathBuf>,

    #[arg(
        long = "tls-version",
        env = "BROKER_TLS_VERSION",
        help_heading = "TLS",
        help = "TLS version to used (default: all)"
    )]
    pub tls_version: Option<TlsVersion>,

    #[command(flatten)]
    pub last_will: Option<LastWillConfigArgs>,
}

impl MqttBrokerConnectArgs {
    fn merge(self, other: MqttBrokerConnect) -> Result<MqttBrokerConnect, ArgsError> {
        let mut builder = MqttBrokerConnectBuilder::default();

        builder.host(match &self.host {
            Some(host) => host.to_string(),
            None => other.host,
        });

        builder.port(match self.port {
            Some(port) => port,
            None => other.port,
        });

        builder.protocol(match &self.protocol {
            Some(protocol) => protocol.into(),
            None => other.protocol,
        });

        builder.client_id(match &self.client_id {
            Some(client_id) => client_id.to_string(),
            None => other.client_id,
        });

        builder.mqtt_version(match &self.mqtt_version {
            Some(mqtt_version) => mqtt_version.into(),
            None => other.mqtt_version,
        });

        builder.keep_alive(match self.keep_alive {
            Some(keep_alive) => keep_alive,
            None => other.keep_alive,
        });

        builder.username(match &self.username {
            Some(username) => Some(username.to_string()),
            None => other.username,
        });

        builder.password(match &self.password {
            Some(password) => Some(password.to_string()),
            None => other.password,
        });

        builder.use_tls(match self.use_tls {
            Some(use_tls) => use_tls,
            None => other.use_tls,
        });

        builder.tls_ca_file(match &self.tls_ca_file {
            Some(tls_ca_file) => Some(PathBuf::from(tls_ca_file)),
            None => other.tls_ca_file,
        });

        builder.tls_client_certificate(match &self.tls_client_certificate {
            Some(tls_client_certificate) => Some(PathBuf::from(tls_client_certificate)),
            None => other.tls_client_certificate,
        });

        builder.tls_client_key(match &self.tls_client_key {
            Some(tls_client_key) => Some(PathBuf::from(tls_client_key)),
            None => other.tls_client_key,
        });

        builder.tls_version(match &self.tls_version {
            Some(tls_version) => tls_version.into(),
            None => other.tls_version,
        });

        builder.last_will(match self.last_will {
            Some(last_will_args) => {
                if let Some(last_will) = other.last_will {
                    Some(last_will_args.merge(last_will)?)
                } else {
                    Some(last_will_args.merge(LastWillConfig::default())?)
                }
            }
            None => other.last_will,
        });

        builder.build().map_err(ArgsError::from)
    }
}

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct LastWillConfigArgs {
    #[arg(
        long = "last-will-payload",
        env = "BROKER_LAST_WILL_PAYLOAD",
        help_heading = "Last will",
        help = "The UTF-8 encoded payload of the will message (default: empty)"
    )]
    pub payload: Option<String>,

    #[arg(
        long = "last-will-topic",
        env = "BROKER_LAST_WILL_TOPIC",
        help_heading = "Last will",
        help = "The topic where the last will message will be published (default: empty)"
    )]
    pub topic: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_qos_option")]
    #[arg(long = "last-will-qos", env = "BROKER_LAST_WILL_QOS", value_parser = parse_qos,
    help_heading = "Last will",
    help = "Quality of Service (default: 0) (possible values: 0 = at most once; 1 = at least once; 2 = exactly once)"
    )
    ]
    pub qos: Option<QoS>,

    #[arg(
        long = "last-will-retain",
        env = "BROKER_LAST_WILL_RETAIN",
        help_heading = "Last will",
        help = "If true, last will message will be retained, else not (default: false)"
    )]
    pub retain: Option<bool>,
}

impl LastWillConfigArgs {
    fn merge(self, other: LastWillConfig) -> Result<LastWillConfig, ArgsError> {
        let mut lw = LastWillConfigBuilder::default();

        lw.topic(match self.topic {
            Some(topic) => topic,
            None => other.topic,
        });
        lw.qos(match self.qos {
            Some(qos) => qos,
            None => other.qos,
        });
        lw.payload(match self.payload {
            Some(payload) => payload.into_bytes(),
            None => other.payload,
        });
        lw.retain(match self.retain {
            Some(retain) => retain,
            None => other.retain,
        });

        lw.build().map_err(ArgsError::from)
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
pub enum TlsVersion {
    #[default]
    #[clap(name = "all")]
    All,
    #[clap(name = "v12")]
    Version1_2,
    #[clap(name = "v13")]
    Version1_3,
}

impl From<TlsVersion> for mqtlib::config::mqtli_config::TlsVersion {
    fn from(value: TlsVersion) -> Self {
        match value {
            TlsVersion::All => Self::All,
            TlsVersion::Version1_2 => Self::Version1_2,
            TlsVersion::Version1_3 => Self::Version1_3,
        }
    }
}

impl From<&TlsVersion> for mqtlib::config::mqtli_config::TlsVersion {
    fn from(value: &TlsVersion) -> Self {
        match value {
            TlsVersion::All => Self::All,
            TlsVersion::Version1_2 => Self::Version1_2,
            TlsVersion::Version1_3 => Self::Version1_3,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
pub enum MqttVersion {
    #[clap(name = "v311")]
    V311,

    #[default]
    #[clap(name = "v5")]
    V5,
}

impl From<MqttVersion> for mqtlib::config::mqtli_config::MqttVersion {
    fn from(value: MqttVersion) -> Self {
        match value {
            MqttVersion::V311 => Self::V311,
            MqttVersion::V5 => Self::V5,
        }
    }
}

impl From<&MqttVersion> for mqtlib::config::mqtli_config::MqttVersion {
    fn from(value: &MqttVersion) -> Self {
        match value {
            MqttVersion::V311 => Self::V311,
            MqttVersion::V5 => Self::V5,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
pub enum MqttProtocol {
    #[default]
    #[clap(name = "tcp")]
    Tcp,

    #[clap(name = "websocket")]
    Websocket,
}

impl From<MqttProtocol> for mqtlib::config::mqtli_config::MqttProtocol {
    fn from(value: MqttProtocol) -> Self {
        match value {
            MqttProtocol::Tcp => Self::Tcp,
            MqttProtocol::Websocket => Self::Websocket,
        }
    }
}

impl From<&MqttProtocol> for mqtlib::config::mqtli_config::MqttProtocol {
    fn from(value: &MqttProtocol) -> Self {
        match value {
            MqttProtocol::Tcp => Self::Tcp,
            MqttProtocol::Websocket => Self::Websocket,
        }
    }
}

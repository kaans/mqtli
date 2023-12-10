use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser};
use derive_getters::Getters;
use log::LevelFilter;


#[derive(Parser, Debug, Getters)]
#[command(author, version, about, long_about = None)]
pub struct MqtliArgs {
    #[command(flatten)]
    broker: MqttBrokerConnectArgs,

    #[command(flatten)]
    logger: LoggingArgs,

    #[arg(long = "config-file", default_value = "config.yaml", env = "CONFIG_FILE_PATH")]
    config_file: PathBuf,

    subscribe_topics: Vec<String>
}

#[derive(Args, Debug, Default, Getters)]
#[group(required = false, multiple = true)]
pub struct MqttBrokerConnectArgs {
    #[arg(short = 'o', long = "host", env = "BROKER_HOST")]
    host: Option<String>,

    #[arg(short = 'p', long = "port", env = "BROKER_PORT")]
    port: Option<u16>,

    #[arg(short = 'c', long = "client-id",  env = "BROKER_CLIENT_ID")]
    client_id: Option<String>,

    #[arg(long = "keep-alive", env = "BROKER_KEEP_ALIVE", value_parser = parse_keep_alive)]
    keep_alive: Option<Duration>,

    #[arg(short = 'u', long = "username", env = "BROKER_USERNAME")]
    username: Option<String>,

    #[arg(short = 'w', long = "password", env = "BROKER_PASSWORD")]
    password: Option<String>,

    #[arg(long = "use-tls", env = "BROKER_USE_TLS")]
    use_tls: Option<bool>,

    #[arg(long = "ca-file", env = "BROKER_TLS_CA_FILE")]
    tls_ca_file: Option<PathBuf>,

    #[arg(long = "client-cert", env = "BROKER_TLS_CLIENT_CERTIFICATE_FILE")]
    tls_client_certificate: Option<PathBuf>,

    #[arg(long = "client-key", env = "BROKER_TLS_CLIENT_KEY_FILE")]
    tls_client_key: Option<PathBuf>,
}

#[derive(Args, Debug, Getters)]
#[group(required = false, multiple = true)]
pub struct LoggingArgs {
    #[arg(short = 'l', long = "log-level", env = "LOG_LEVEL")]
    level: Option<LevelFilter>,
}

fn parse_keep_alive(input: &str) -> Result<Duration, String> {
    let duration_in_seconds: u64 = input.parse()
        .map_err(|_| format!("{input} is not a valid duration in seconds"))?;

    Ok(Duration::from_secs(duration_in_seconds))
}

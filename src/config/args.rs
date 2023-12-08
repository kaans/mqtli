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
    #[arg(short = 'o', long = "host", env = "HOST")]
    host: Option<String>,

    #[arg(short = 'p', long = "port", env = "PORT")]
    port: Option<u16>,

    #[arg(short = 'c', long = "client-id",  env = "CLIENT_ID")]
    client_id: Option<String>,

    #[arg(long = "keep-alive", env = "KEEP_ALIVE", value_parser = parse_keep_alive)]
    keep_alive: Option<Duration>,

    #[arg(short = 'u', long = "username", env = "USERNAME")]
    username: Option<String>,

    #[arg(short = 'w', long = "password", env = "PASSWORD")]
    password: Option<String>,

    //#[arg(long = "use-tls", default_value_t = false, env = "USE_TLS")]
    //use_tls: bool,
//
    //#[arg(long = "verify-ca", default_value_t = true, env = "TLS_VERIFY_CA", requires = "use_tls")]
    //tls_verify_ca: bool,
//
    //#[arg(long = "ca-file", env = "TLS_CA_FILE", requires = "tls_verify_ca")]
    //tls_ca_file: Option<PathBuf>,
//
    //#[arg(long = "client-cert", env = "TLS_CLIENT_CERTIFICATE_FILE", requires = "tls_client_key")]
    //tls_client_certificate: Option<PathBuf>,
//
    //#[arg(long = "client-key", env = "TLS_CLIENT_KEY_FILE", requires = "tls_client_certificate")]
    //tls_client_key: Option<PathBuf>,
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

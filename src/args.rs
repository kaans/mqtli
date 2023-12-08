use std::fmt::Debug;
use std::time::Duration;

use clap::Args;
use derive_getters::Getters;
use log::LevelFilter;

#[derive(Args, Debug, Default, Getters)]
#[group(required = false, multiple = true)]
pub struct MqttBrokerConnectArgs {
    #[arg(short = 'o', long = "host", default_value = "localhost", env = "HOST")]
    host: String,

    #[arg(short = 'p', long = "port", default_value_t = 1883, env = "PORT")]
    port: u16,

    #[arg(short = 'c', long = "client-id", default_value = "mqtli", env = "CLIENT_ID")]
    client_id: String,

    #[arg(long = "keep-alive", default_value = "5", env = "KEEP_ALIVE", value_parser = parse_keep_alive)]
    keep_alive: Duration,

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
    #[arg(short = 'l', long = "log-level", default_value_t = LevelFilter::Info, env = "LOG_LEVEL")]
    level: LevelFilter,
}

fn parse_keep_alive(input: &str) -> Result<Duration, String> {
    let duraton_in_seconds: u64 = input.parse()
        .map_err(|_| format!("{input} is not a valid duration in seconds"))?;

    Ok(Duration::from_secs(duraton_in_seconds))
}
use clap::Args;
use derive_getters::Getters;
use serde::Deserialize;

#[derive(Args, Debug, Default, Deserialize, Getters)]
pub struct SqlStorage {
    #[arg(
        long = "connection-string",
        env = "SQL_CONNECTION_STRING",
        global = true,
        help_heading = "SQL storage",
        help = "The connection string to the SQL storage (currently only sqlite is supported)"
    )]
    #[serde(rename = "connection_string")]
    pub connection_string: String,
}

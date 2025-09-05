use crate::config::mqtli_config::MqtliConfig;
use crate::storage::{get_sql_storage, SqlStorageError, SqlStorageImpl};
use thiserror::Error;

pub mod config;
pub mod mqtt;
pub mod output;
pub mod payload;
pub mod publish;
pub mod sparkplug;
pub mod storage;

#[derive(Error, Debug)]
pub enum MqtlibError {
    #[error("SQL storage error")]
    SqlStorageError(#[from] SqlStorageError),
}

#[derive(Debug, Default)]
pub struct Mqtlib {
    config: MqtliConfig,
    sql_storage: Option<Box<dyn SqlStorageImpl>>,
}

impl Mqtlib {
    pub fn new(config: MqtliConfig) -> Self {
        Self {
            config,
            sql_storage: None,
        }
    }

    pub async fn init(&mut self) -> Result<(), MqtlibError> {
        if let Some(sql) = self.config.sql_storage.as_ref() {
            self.sql_storage = Some(get_sql_storage(sql).await?);
        }

        Ok(())
    }
}

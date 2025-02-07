use crate::storage::sqlite::SqlStorageSqlite;
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;
use crate::mqtt::QoS;
use crate::payload::{PayloadFormat, PayloadFormatError};

pub mod sqlite;

#[derive(Debug, Error)]
pub enum SqlStorageError {
    #[error("Unsupported SQL database with scheme {0}")]
    UnsupportedSqlDatabase(String),
    #[error("Error while connecting to database")]
    SqlConnectionError(#[from] sqlx::Error),
    #[error("Error while formatting payload")]
    PayloadFormatError(#[from] PayloadFormatError),
}

#[async_trait]
pub trait SqlStorageImpl: Debug + Send + Sync {
    async fn insert(&self, statement: &str, topic: &str, qos: QoS, retain: bool, payload: &PayloadFormat) -> Result<u64, SqlStorageError>;
    async fn execute(&self, statement: &str) -> Result<u64, SqlStorageError>;
}

pub async fn get_sql_storage(
    sql: &crate::config::sql_storage::SqlStorage,
) -> Result<Box<dyn SqlStorageImpl>, SqlStorageError> {
    match sql.scheme().as_str() {
        "sqlite" => {
            let opts = SqliteConnectOptions::from_str(sql.connection_string.as_str())?
                .journal_mode(SqliteJournalMode::Wal)
                .read_only(false);

            let db = SqlStorageSqlite::new(SqlitePool::connect_with(opts).await?);

            Ok(Box::new(db))
        }
        scheme => Err(SqlStorageError::UnsupportedSqlDatabase(scheme.to_string())),
    }
}

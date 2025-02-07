use crate::storage::sql_storage::SqlStorageSqlite;
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;

pub mod sql_storage;

#[derive(Debug, Error)]
pub enum SqlStorageError {
    #[error("Unsupported SQL database with scheme {0}")]
    UnsupportedSqlDatabase(String),
    #[error("Error while connecting to database")]
    SqlConnectionError(#[from] sqlx::Error),
}

#[async_trait]
pub trait SqlStorageImpl: Debug {
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

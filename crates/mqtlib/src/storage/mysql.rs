use crate::storage::{SqlStorageError, SqlStorageImpl};
use async_trait::async_trait;
use sqlx::MySqlPool;
use std::fmt::Debug;
use crate::mqtt::QoS;
use crate::payload::PayloadFormat;

#[derive(Debug)]
pub struct SqlStorageMySql {
    pool: MySqlPool,
}

impl SqlStorageMySql {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SqlStorageImpl for SqlStorageMySql {
    async fn insert(&self, statement: &str, topic: &str, qos: QoS, retain: bool, payload: &PayloadFormat) -> Result<u64, SqlStorageError> {
        let query_statement = statement
            .replace("{{topic}}", topic)
            .replace("{{retain}}", if retain {"1"} else {"0"})
            .replace("{{qos}}", (qos as i32).to_string().as_ref())
            .replace("{{payload}}", "?");

        let payload = Vec::<u8>::try_from(payload.clone())?;

        let mut query = sqlx::query(query_statement.as_ref());
        if statement.contains("{{payload}}") {
            query = query.bind(payload);
        }

        let result = query.execute(&self.pool).await;

        Ok(result?.rows_affected())
    }

    async fn execute(&self, statement: &str) -> Result<u64, SqlStorageError> {
        let result = sqlx::query(statement).execute(&self.pool).await;
        Ok(result?.rows_affected())
    }
}

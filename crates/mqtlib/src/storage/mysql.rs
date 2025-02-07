use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use crate::storage::{SqlStorageError, SqlStorageImpl};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::MySqlPool;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

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
    async fn insert(
        &self,
        statement: &str,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload: &PayloadFormat,
    ) -> Result<u64, SqlStorageError> {
        let query_statement = statement
            .replace("{{topic}}", topic)
            .replace("{{retain}}", if retain { "1" } else { "0" })
            .replace("{{qos}}", (qos as i32).to_string().as_ref())
            .replace(
                "{{created_at}}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string()
                    .as_ref(),
            )
            .replace(
                "{{created_at_millis}}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
                    .as_ref(),
            )
            .replace(
                "{{created_at_iso}}",
                Utc::now()
                    .format("%Y-%m-%d %H:%M:%S%.6f")
                    .to_string()
                    .as_str(),
            )
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

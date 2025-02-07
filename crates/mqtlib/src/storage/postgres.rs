use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use crate::storage::{SqlStorageError, SqlStorageImpl};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct SqlStoragePostgres {
    pool: PgPool,
}

impl SqlStoragePostgres {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SqlStorageImpl for SqlStoragePostgres {
    async fn insert(
        &self,
        statement: &str,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload: &PayloadFormat,
    ) -> Result<u64, SqlStorageError> {
        let query = statement
            .replace("{{topic}}", topic)
            .replace("{{retain}}", if retain { "true" } else { "false" })
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
            .replace("{{created_at_iso}}", Utc::now().to_rfc3339().as_str())
            .replace("{{payload}}", "$1");

        let payload = Vec::<u8>::try_from(payload.clone())?;

        let result = sqlx::query(query.as_ref())
            .bind(payload)
            .execute(&self.pool)
            .await;

        Ok(result?.rows_affected())
    }

    async fn execute(&self, statement: &str) -> Result<u64, SqlStorageError> {
        let result = sqlx::query(statement).execute(&self.pool).await;
        Ok(result?.rows_affected())
    }
}

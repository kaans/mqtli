use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use crate::storage::{SqlStorageError, SqlStorageImpl};
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

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
        let mut queries: Vec<(String, Vec<Vec<u8>>)> = vec![];

        self.create_queries(statement, topic, qos, retain, payload, &mut queries)?;

        let mut affected_rows = 0;
        for (query, binds) in queries {
            let mut result = sqlx::query(query.as_ref());
            for bind in binds {
                result = result.bind(bind);
            }
            let result = result.execute(&self.pool).await;
            affected_rows += result?.rows_affected();
        }
        Ok(affected_rows)
    }

    async fn execute(&self, statement: &str) -> Result<u64, SqlStorageError> {
        let result = sqlx::query(statement).execute(&self.pool).await;
        Ok(result?.rows_affected())
    }

    fn get_placeholder(&self, counter: usize) -> String {
        format!("${}", counter)
    }
}

use crate::mqtt::QoS;
use crate::payload::PayloadFormat;
use crate::storage::{SqlStorageError, SqlStorageImpl};
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::fmt::Debug;

#[derive(Debug)]
pub struct SqlStorageSqlite {
    pool: SqlitePool,
}

impl SqlStorageSqlite {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SqlStorageImpl for SqlStorageSqlite {
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
            .replace("{{retain}}", if retain { "1" } else { "0" })
            .replace("{{qos}}", (qos as i32).to_string().as_ref())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::text::PayloadFormatText;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
    use sqlx::Row;
    use std::str::FromStr;

    const CREATE_TABLE: &str = "
CREATE TABLE test (
    id  INTEGER PRIMARY KEY AUTOINCREMENT,
    topic TEXT NOT NULL,
    qos INTEGER NOT NULL,
    retain INTEGER NOT NULL,
    payload BLOB NULL
);";

    const INSERT: &str = "
INSERT INTO test
(topic, qos, retain, payload)
VALUES
(\"{topic}\", \"{qos}\", \"{retain}\", \"{payload}\");
";

    #[tokio::test]
    async fn insert() {
        let db = get_db().await;

        let result = db
            .insert(
                INSERT,
                "topic",
                QoS::AtLeastOnce,
                false,
                &PayloadFormat::Text(PayloadFormatText {
                    content: "PAYLOAD".as_bytes().to_vec(),
                }),
            )
            .await;
        assert!(result.is_ok());

        print_table_content(&db).await;
    }

    async fn get_db() -> SqlStorageSqlite {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .journal_mode(SqliteJournalMode::Wal)
            .read_only(false);

        let db = SqlStorageSqlite {
            pool: SqlitePool::connect_with(opts).await.unwrap(),
        };
        assert!(db.execute(CREATE_TABLE).await.is_ok());
        db
    }

    async fn print_table_content(db: &SqlStorageSqlite) {
        let result = sqlx::query("SELECT * FROM test")
            .fetch_all(&db.pool)
            .await
            .unwrap();
        for r in result {
            println!(
                "{} - {} - {} - {}",
                r.get::<String, &str>("topic"),
                r.get::<String, &str>("qos"),
                r.get::<String, &str>("retain"),
                r.get::<String, &str>("payload")
            )
        }
    }
}

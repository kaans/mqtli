use crate::mqtt::QoS;
use crate::payload::sparkplug::protos::sparkplug_b::payload::metric::Value;
use crate::payload::PayloadFormat;
use crate::sparkplug::topic::SparkplugTopic;
use crate::storage::{SqlStorageError, SqlStorageImpl};
use async_trait::async_trait;
use chrono::Utc;
use protobuf::Message;
use sqlx::SqlitePool;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct SqlStorageSqlite {
    pool: SqlitePool,
}

impl SqlStorageSqlite {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn replace_basic_properties(
        statement: &str,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload: Vec<u8>,
        binds: &mut Vec<Vec<u8>>,
    ) -> String {
        let query = statement
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
                    .format("%Y-%m-%d %H:%M:%S%.3f")
                    .to_string()
                    .as_str(),
            )
            .replace("{{payload}}", format!("${}", binds.len() + 1).as_str());

        binds.push(payload);

        query
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
        let mut queries: Vec<(String, Vec<Vec<u8>>)> = vec![];

        match payload {
            PayloadFormat::Sparkplug(sp) => {
                let sp_topic = SparkplugTopic::try_from(topic)?;
                let payload = Vec::<u8>::try_from(payload.clone())?;

                match sp_topic {
                    SparkplugTopic::EdgeNode(sp_topic) => {
                        let device_id = sp_topic.device_id.unwrap_or(String::from(""));

                        for metric in &sp.content.metrics {
                            let mut binds: Vec<Vec<u8>> = vec![];
                            let mut query = Self::replace_basic_properties(
                                statement,
                                topic,
                                qos,
                                retain,
                                payload.clone(),
                                &mut binds,
                            );

                            query = query.replace("{{sp_version}}", sp_topic.version.as_str());
                            query = query.replace(
                                "{{sp_message_type}}",
                                sp_topic.message_type.to_string().as_str(),
                            );
                            query = query.replace("{{sp_group_id}}", sp_topic.group_id.as_str());
                            query = query
                                .replace("{{sp_edge_node_id}}", sp_topic.edge_node_id.as_str());
                            query = query.replace("{{sp_device_id}}", device_id.as_str());
                            query = query.replace(
                                "{{sp_metric_level}}",
                                sp_topic.metric_levels.join("/").as_str(),
                            );
                            query = query.replace(
                                "{{sp_metric_name}}",
                                metric.name.as_ref().unwrap_or(&"".to_string()),
                            );

                            let value: Vec<u8> = match &metric.value {
                                None => vec![],
                                Some(value) => match value {
                                    Value::IntValue(value) => value.to_string().into_bytes(),
                                    Value::LongValue(value) => value.to_string().into_bytes(),
                                    Value::FloatValue(value) => value.to_string().into_bytes(),
                                    Value::DoubleValue(value) => value.to_string().into_bytes(),
                                    Value::BooleanValue(value) => value.to_string().into_bytes(),
                                    Value::StringValue(value) => value.clone().into_bytes(),
                                    Value::BytesValue(value) => value.clone(),
                                    Value::DatasetValue(value) => {
                                        value.write_to_bytes().unwrap_or(vec![])
                                    }
                                    Value::TemplateValue(value) => {
                                        value.write_to_bytes().unwrap_or(vec![])
                                    }
                                    Value::ExtensionValue(value) => {
                                        value.write_to_bytes().unwrap_or(vec![])
                                    }
                                },
                            };

                            query = query.replace(
                                "{{sp_metric_value}}",
                                format!("${}", binds.len() + 1).as_str(),
                            );
                            binds.push(value);

                            queries.push((query, binds));
                        }
                    }
                    SparkplugTopic::HostApplication(sp_topic) => {
                        let mut binds: Vec<Vec<u8>> = vec![];
                        let mut query = Self::replace_basic_properties(
                            statement,
                            topic,
                            qos,
                            retain,
                            payload.clone(),
                            &mut binds,
                        );

                        query = query.replace("{{sp_version}}", sp_topic.version.as_str());
                        query = query.replace(
                            "{{sp_message_type}}",
                            sp_topic.message_type.to_string().as_str(),
                        );
                        query = query.replace("{{sp_host_id}}", sp_topic.host_id.as_str());

                        queries.push((query, binds));
                    }
                }
            }
            _ => {
                let mut binds: Vec<Vec<u8>> = vec![];
                let query = Self::replace_basic_properties(
                    statement,
                    topic,
                    qos,
                    retain,
                    Vec::<u8>::try_from(payload.clone())?,
                    &mut binds,
                );
                queries.push((query, binds));
            }
        }

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

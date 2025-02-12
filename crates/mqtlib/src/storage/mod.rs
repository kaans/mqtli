use crate::mqtt::QoS;
use crate::payload::sparkplug::protos::sparkplug_b::payload::metric::Value;
use crate::payload::{PayloadFormat, PayloadFormatError};
use crate::sparkplug::topic::SparkplugTopic;
use crate::sparkplug::SparkplugError;
use crate::storage::mysql::SqlStorageMySql;
use crate::storage::postgres::SqlStoragePostgres;
use crate::storage::sqlite::SqlStorageSqlite;
use async_trait::async_trait;
use chrono::Utc;
use protobuf::Message;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{MySqlPool, PgPool, SqlitePool};
use std::fmt::Debug;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::warn;

pub mod mysql;
mod postgres;
pub mod sqlite;

#[derive(Debug, Error)]
pub enum SqlStorageError {
    #[error("Unsupported SQL database with scheme {0}")]
    UnsupportedSqlDatabase(String),
    #[error("Error while connecting to database")]
    SqlConnectionError(#[from] sqlx::Error),
    #[error("Error while formatting payload")]
    PayloadFormatError(#[from] PayloadFormatError),
    #[error("Error in Sparkplug format")]
    SparkplugError(#[from] SparkplugError),
}

#[async_trait]
pub trait SqlStorageImpl: Debug + Send + Sync {
    async fn insert(
        &self,
        statement: &str,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload: &PayloadFormat,
    ) -> Result<u64, SqlStorageError>;
    async fn execute(&self, statement: &str) -> Result<u64, SqlStorageError>;

    fn get_placeholder(&self, usize: usize) -> String;

    fn replace_basic_properties(
        &self,
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
            .replace(
                "{{payload}}",
                self.get_placeholder(binds.len() + 1).as_str(),
            );

        binds.push(payload);

        query
    }

    fn create_queries(
        &self,
        statement: &str,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload_input: &PayloadFormat,
        queries: &mut Vec<(String, Vec<Vec<u8>>)>,
    ) -> Result<(), SqlStorageError> {
        let payload_output = Vec::<u8>::try_from(payload_input.clone())?;

        match payload_input {
            PayloadFormat::Sparkplug(sp) => {
                let sp_topic = SparkplugTopic::try_from(topic)?;

                if let SparkplugTopic::EdgeNode(sp_topic) = sp_topic {
                    let device_id = sp_topic.device_id.unwrap_or(String::from(""));

                    for metric in &sp.content.metrics {
                        let mut binds: Vec<Vec<u8>> = vec![];
                        let mut query = self.replace_basic_properties(
                            statement,
                            topic,
                            qos,
                            retain,
                            payload_output.clone(),
                            &mut binds,
                        );

                        query = query.replace("{{sp_version}}", sp_topic.version.as_str());
                        query = query.replace(
                            "{{sp_message_type}}",
                            sp_topic.message_type.to_string().as_str(),
                        );
                        query = query.replace("{{sp_group_id}}", sp_topic.group_id.as_str());
                        query =
                            query.replace("{{sp_edge_node_id}}", sp_topic.edge_node_id.as_str());
                        query = query.replace("{{sp_device_id}}", device_id.as_str());
                        query = query.replace(
                            "{{sp_metric_level}}",
                            (if !sp_topic.metric_levels.is_empty() {
                                format!("'{}'", sp_topic.metric_levels.join("/"))
                            } else {
                                "null".to_string()
                            })
                            .as_str(),
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
                            self.get_placeholder(binds.len() + 1).as_str(),
                        );
                        binds.push(value);

                        queries.push((query, binds));
                    }
                } else {
                    warn!("Received Sparkplug payload on a host application topic ({}) which is not supported. \
                    The payload must be of type sparkplug JSON.",
                        topic
                    )
                }
            }
            PayloadFormat::SparkplugJson(sp) => {
                let sp_topic = SparkplugTopic::try_from(topic)?;
                if let SparkplugTopic::HostApplication(sp_topic) = sp_topic {
                    let mut binds: Vec<Vec<u8>> = vec![];

                    let mut query = self.replace_basic_properties(
                        statement,
                        topic,
                        qos,
                        retain,
                        payload_output.clone(),
                        &mut binds,
                    );

                    query = query.replace("{{sp_version}}", sp_topic.version.as_str());
                    query = query.replace(
                        "{{sp_message_type}}",
                        sp_topic.message_type.to_string().as_str(),
                    );
                    query = query.replace("{{sp_host_id}}", sp_topic.host_id.as_str());

                    let online = sp.content().get("online");
                    if online.is_none() {
                        warn!(
                            "Required attribute \"online\" not found in payload of STATE message"
                        );
                    }
                    query = query.replace(
                        "{{sp_host_online}}",
                        online
                            .unwrap_or(&serde_json::Value::String("".to_string()))
                            .as_str()
                            .unwrap(),
                    );

                    let timestamp = sp.content().get("timestamp");
                    if timestamp.is_none() {
                        warn!("Required attribute \"timestamp\" not found in payload of STATE message");
                    }
                    query = query.replace(
                        "{{sp_host_timestamp}}",
                        timestamp
                            .unwrap_or(&serde_json::Value::String("".to_string()))
                            .as_str()
                            .unwrap(),
                    );

                    queries.push((query, binds));
                } else {
                    warn!("Received Sparkplug JSON payload on an edge node topic ({}) which is not supported. \
                        The payload must be of type (binary) sparkplug.",
                        topic
                    )
                }
            }
            _ => {
                let mut binds: Vec<Vec<u8>> = vec![];
                let query = self.replace_basic_properties(
                    statement,
                    topic,
                    qos,
                    retain,
                    payload_output,
                    &mut binds,
                );
                queries.push((query, binds));
            }
        }
        Ok(())
    }
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
        "mysql" | "mariadb" => {
            let opts = MySqlConnectOptions::from_str(sql.connection_string.as_str())?;

            let db = SqlStorageMySql::new(MySqlPool::connect_with(opts).await?);

            Ok(Box::new(db))
        }
        "postgresql" => {
            let opts = PgConnectOptions::from_str(sql.connection_string.as_str())?;

            let db = SqlStoragePostgres::new(PgPool::connect_with(opts).await?);

            Ok(Box::new(db))
        }
        scheme => Err(SqlStorageError::UnsupportedSqlDatabase(scheme.to_string())),
    }
}

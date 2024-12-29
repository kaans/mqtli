use log::LevelFilter;
use mqtlib::config::deserialize_qos;
use mqtlib::mqtt::QoS;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use std::time::Duration;

pub fn deserialize_duration_seconds<'a, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'a>,
{
    let value: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Some(Duration::from_secs(value)))
}

pub fn deserialize_qos_option<'a, D>(deserializer: D) -> Result<Option<QoS>, D::Error>
where
    D: Deserializer<'a>,
{
    Ok(Some(deserialize_qos(deserializer)?))
}

pub fn parse_keep_alive(input: &str) -> Result<Duration, String> {
    let duration_in_seconds: u64 = input
        .parse()
        .map_err(|_| format!("{input} is not a valid duration in seconds"))?;

    Ok(Duration::from_secs(duration_in_seconds))
}

pub fn parse_qos(input: &str) -> Result<QoS, String> {
    let qos: QoS = match input {
        "0" => QoS::AtMostOnce,
        "1" => QoS::AtLeastOnce,
        "2" => QoS::ExactlyOnce,
        _ => return Err("QoS value must be 0, 1 or 2".to_string()),
    };

    Ok(qos)
}

pub fn deserialize_level_filter<'a, D>(deserializer: D) -> Result<Option<LevelFilter>, D::Error>
where
    D: Deserializer<'a>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;

    let level = match LevelFilter::from_str(value) {
        Ok(level) => level,
        Err(_) => {
            return Err(Error::invalid_value(
                Unexpected::Other(value),
                &"unsigned integer between 0 and 2",
            ));
        }
    };

    Ok(Some(level))
}

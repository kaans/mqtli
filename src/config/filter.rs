use std::str::FromStr;
use serde::Deserialize;
use derive_getters::Getters;
use jsonpath_rust::{JsonPath, JsonPathParserError};
use crate::payload::PayloadFormat;
use thiserror::Error;
use crate::payload::json::PayloadFormatJson;

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Payload has wrong format, expected format `{0}`")]
    WrongPayloadFormat(String),
    #[error("The given JSON path cannot be parsed")]
    WrongJsonPath(#[from] JsonPathParserError),
}

pub trait FilterImpl {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError>;
}

#[derive(Clone, Debug, Deserialize, Getters, PartialEq)]
pub struct FilterTypeExtractJson {
    jsonpath: String,
    #[serde(rename = "ignore_non_json")]
    #[serde(default)]
    ignore_none_json_payload: bool
}

impl Default for FilterTypeExtractJson {
    fn default() -> Self {
        Self {
            jsonpath: "".to_string(),
            ignore_none_json_payload: false,
        }
    }
}

impl FilterImpl for FilterTypeExtractJson {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        let result: Result<Vec<PayloadFormat>, FilterError> = match data {
            PayloadFormat::Json(data) => {
                let result: Result<Vec<PayloadFormat>, FilterError> = match JsonPath::from_str(self.jsonpath.as_str()) {
                    Ok(path) => {
                        let res: Vec<PayloadFormat> = path.find_slice(data.content()).iter().map(|v| {
                            PayloadFormat::Json(PayloadFormatJson::from(v.clone().to_data()))
                        }).collect();

                        Ok(res)
                    }
                    Err(e) => {
                        return Err(FilterError::WrongJsonPath(e));
                    }
                };

                result
            }
            data => {
                if self.ignore_none_json_payload {
                    Ok(vec![data])
                } else {
                    Err(FilterError::WrongPayloadFormat("json".into()))
                }
            }
        };

        result
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum FilterType {
    #[serde(rename = "extract_json")]
    ExtractJson(FilterTypeExtractJson)
}

impl Default for FilterType {
    fn default() -> Self {
        Self::ExtractJson(FilterTypeExtractJson::default())
    }
}

impl FilterImpl for FilterType {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        match self {
            FilterType::ExtractJson(filter) => filter.apply(data)
        }
    }
}
use crate::payload::json::PayloadFormatJson;
use crate::payload::text::PayloadFormatText;
use crate::payload::{PayloadFormat, PayloadFormatError};
use derive_getters::Getters;
use jsonpath_rust::{JsonPath, JsonPathParserError};
use serde::Deserialize;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Payload has wrong format, expected format `{0}`")]
    WrongPayloadFormat(String),
    #[error("The given JSON path cannot be parsed")]
    WrongJsonPath(#[from] JsonPathParserError),
    #[error("Error in payload format")]
    PayloadFormatError(#[from] PayloadFormatError),
}

pub trait FilterImpl {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError>;
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeExtractJson {
    jsonpath: String,
    #[serde(rename = "ignore_non_json")]
    #[serde(default)]
    ignore_none_json_payload: bool,
}

impl FilterImpl for FilterTypeExtractJson {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        let result: Result<Vec<PayloadFormat>, FilterError> = match data {
            PayloadFormat::Json(data) => {
                let result: Result<Vec<PayloadFormat>, FilterError> =
                    match JsonPath::from_str(self.jsonpath.as_str()) {
                        Ok(path) => {
                            let res: Vec<PayloadFormat> = path
                                .find_slice(data.content())
                                .iter()
                                .map(|v| {
                                    PayloadFormat::Json(PayloadFormatJson::from(
                                        v.clone().to_data(),
                                    ))
                                })
                                .collect();

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

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeToUpperCase {
    #[serde(rename = "ignore_non_text")]
    #[serde(default)]
    ignore_none_text_payload: bool,
}

impl FilterImpl for FilterTypeToUpperCase {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        let result: Result<Vec<PayloadFormat>, FilterError> = match data {
            PayloadFormat::Text(data) => {
                let res = PayloadFormatText::from(data.content().to_ascii_uppercase());
                Ok(vec![PayloadFormat::Text(res)])
            }
            data => {
                if self.ignore_none_text_payload {
                    Ok(vec![data])
                } else {
                    Err(FilterError::WrongPayloadFormat("text".into()))
                }
            }
        };

        result
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeToText {}

impl FilterImpl for FilterTypeToText {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        Ok(vec![PayloadFormat::Text(
            PayloadFormatText::try_from(data)
                .map_err(FilterError::PayloadFormatError)?,
        )])
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeToJson {}

impl FilterImpl for FilterTypeToJson {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        Ok(vec![PayloadFormat::Json(
            PayloadFormatJson::try_from(data)
                .map_err(FilterError::PayloadFormatError)?,
        )])
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum FilterType {
    #[serde(rename = "extract_json")]
    ExtractJson(FilterTypeExtractJson),
    #[serde(rename = "to_upper")]
    ToUpperCase(FilterTypeToUpperCase),
    #[serde(rename = "to_text")]
    ToText(FilterTypeToText),
    #[serde(rename = "to_json")]
    ToJson(FilterTypeToJson),
}

impl Default for FilterType {
    fn default() -> Self {
        Self::ExtractJson(FilterTypeExtractJson::default())
    }
}

impl FilterImpl for FilterType {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        match self {
            FilterType::ExtractJson(filter) => filter.apply(data),
            FilterType::ToUpperCase(filter) => filter.apply(data),
            FilterType::ToText(filter) => filter.apply(data),
            FilterType::ToJson(filter) => filter.apply(data),
        }
    }
}

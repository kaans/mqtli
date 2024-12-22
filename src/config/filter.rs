use serde::Deserialize;
use derive_getters::Getters;
use crate::payload::PayloadFormat;

pub trait FilterImpl {
    fn apply(&self, data: PayloadFormat) -> anyhow::Result<PayloadFormat>;
}

#[derive(Clone, Debug, Deserialize, Getters, PartialEq)]
pub struct FilterTypeExtractJson {
    jsonpath: String
}

impl Default for FilterTypeExtractJson {
    fn default() -> Self {
        Self {
            jsonpath: "".to_string(),
        }
    }
}

impl FilterImpl for FilterTypeExtractJson {
    fn apply(&self, data: PayloadFormat) -> anyhow::Result<PayloadFormat> {
        Ok(data)
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
    fn apply(&self, data: PayloadFormat) -> anyhow::Result<PayloadFormat> {
        match self {
            FilterType::ExtractJson(filter) => filter.apply(data)
        }
    }
}
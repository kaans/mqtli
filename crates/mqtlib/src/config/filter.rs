use crate::config::PayloadType;
use crate::payload::json::PayloadFormatJson;
use crate::payload::text::PayloadFormatText;
use crate::payload::{PayloadFormat, PayloadFormatError};
use derive_getters::Getters;
use jsonpath_rust::{JsonPath, JsonPathParserError};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Payload has wrong format, expected format `{0}`")]
    WrongPayloadFormat(String),
    #[error("The given JSON path cannot be parsed")]
    WrongJsonPath(#[from] JsonPathParserError),
    #[error("Error in payload format")]
    PayloadFormatError(#[from] Box<PayloadFormatError>),
}

pub trait FilterImpl {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError>;

    fn convert_payload_format(
        &self,
        data: PayloadFormat,
        payload_type: PayloadType,
    ) -> Result<PayloadFormat, FilterError> {
        PayloadFormat::try_from((data, payload_type))
            .map_err(|e| FilterError::PayloadFormatError(Box::new(e)))
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct FilterTypes(pub(crate) Vec<FilterType>);

impl FilterTypes {
    pub fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        self.0.iter().try_fold(vec![data], |payloads, filter| {
            let result: Result<Vec<PayloadFormat>, FilterError> = payloads
                .iter()
                .map(|payload| FilterImpl::apply(filter, payload.clone()))
                .try_fold(vec![], |mut unrolled, result| {
                    unrolled.extend(result?);
                    Ok(unrolled)
                });

            result
        })
    }
}

impl Display for FilterTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0
            .iter()
            .try_for_each(|filter| writeln!(f, "{}", filter))
    }
}

impl From<Vec<FilterType>> for FilterTypes {
    fn from(value: Vec<FilterType>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeExtractJson {
    jsonpath: String,
}

impl FilterImpl for FilterTypeExtractJson {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        let result: Result<Vec<PayloadFormat>, FilterError> =
            match self.convert_payload_format(data, PayloadType::Json)? {
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
                _ => Err(FilterError::WrongPayloadFormat("json".into())),
            };

        result
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeToUpperCase {}

impl FilterImpl for FilterTypeToUpperCase {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        let result: Result<Vec<PayloadFormat>, FilterError> =
            match self.convert_payload_format(data, PayloadType::Text)? {
                PayloadFormat::Text(data) => {
                    let res = PayloadFormatText::from(data.content().to_ascii_uppercase());
                    Ok(vec![PayloadFormat::Text(res)])
                }
                _ => Err(FilterError::WrongPayloadFormat("text".into())),
            };

        result
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeToLowerCase {}

impl FilterImpl for FilterTypeToLowerCase {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        let result: Result<Vec<PayloadFormat>, FilterError> =
            match self.convert_payload_format(data, PayloadType::Text)? {
                PayloadFormat::Text(data) => {
                    let res = PayloadFormatText::from(data.content().to_ascii_lowercase());
                    Ok(vec![PayloadFormat::Text(res)])
                }
                _ => Err(FilterError::WrongPayloadFormat("text".into())),
            };

        result
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeToText {}

impl FilterImpl for FilterTypeToText {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        self.convert_payload_format(data, PayloadType::Text)
            .map(|e| vec![e])
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq)]
pub struct FilterTypeToJson {}

impl FilterImpl for FilterTypeToJson {
    fn apply(&self, data: PayloadFormat) -> Result<Vec<PayloadFormat>, FilterError> {
        self.convert_payload_format(data, PayloadType::Json)
            .map(|e| vec![e])
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, strum_macros::Display)]
#[serde(tag = "type")]
pub enum FilterType {
    #[serde(rename = "extract_json")]
    ExtractJson(FilterTypeExtractJson),
    #[serde(rename = "to_upper")]
    ToUpperCase(FilterTypeToUpperCase),
    #[serde(rename = "to_lower")]
    ToLowerCase(FilterTypeToLowerCase),
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
            FilterType::ToLowerCase(filter) => filter.apply(data),
            FilterType::ToText(filter) => filter.apply(data),
            FilterType::ToJson(filter) => filter.apply(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_text() {
        let filter = FilterTypeToText::default();
        let payload = PayloadFormat::Json(
            PayloadFormatJson::try_from(Vec::from("{\"name\":\"MQTli\"}".as_bytes())).unwrap(),
        );

        let result = filter.apply(payload);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(1, result.len());
        let PayloadFormat::Text(result) = &result[0] else {
            panic!()
        };
        assert_eq!("{\"name\":\"MQTli\"}", result.to_string());
    }

    #[test]
    fn to_json() {
        let filter = FilterTypeToJson::default();
        let payload = PayloadFormat::Text(PayloadFormatText::from("{\"name\":\"MQTli\"}"));

        let result = filter.apply(payload);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(1, result.len());
        let PayloadFormat::Json(result) = &result[0] else {
            panic!()
        };
        assert_eq!("MQTli", result.content().get("name").unwrap());
    }

    #[test]
    fn to_upper() {
        let filter = FilterTypeToUpperCase::default();
        let payload = PayloadFormat::Text(PayloadFormatText::from("MqTli"));

        let result = filter.apply(payload);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(1, result.len());
        let PayloadFormat::Text(result) = &result[0] else {
            panic!()
        };
        assert_eq!("MQTLI", result.to_string());
    }

    #[test]
    fn extract_json() {
        let filter = FilterTypeExtractJson {
            jsonpath: String::from("$.name"),
        };
        let payload = PayloadFormat::Json(
            PayloadFormatJson::try_from(Vec::from("{\"name\":\"MQTli\"}".as_bytes())).unwrap(),
        );

        let result = filter.apply(payload);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(1, result.len());
        let PayloadFormat::Json(result) = &result[0] else {
            panic!()
        };
        assert_eq!("MQTli", result.content());
    }
}

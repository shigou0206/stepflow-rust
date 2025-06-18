use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use stepflow_mapping::model::MappingDSL;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSpec {
    pub r#type: String,
    pub fields: MappingDSL,
    pub inject: Option<InjectTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InjectTarget {
    Header {
        header_name: String,
        format: String,
    },
    Query {
        key: String,
        format: String,
    },
    Body {
        key: String,
        format: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResult {
    pub access_token: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
}

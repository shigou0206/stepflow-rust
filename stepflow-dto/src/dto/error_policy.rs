// stepflow-dto/src/dto/error_policy.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryPolicy {
    pub error_equals: Vec<String>,
    #[serde(default)]
    pub interval_seconds: Option<u32>,
    #[serde(default)]
    pub backoff_rate: Option<f64>,
    #[serde(default)]
    pub max_attempts: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatchPolicy {
    pub error_equals: Vec<String>,
    pub next: String,
    #[serde(default)]
    pub result_path: Option<String>,
}
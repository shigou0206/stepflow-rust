use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimerDto {
    pub timer_id: String,
    pub run_id: String,
    pub shard_id: i64,
    pub fire_at: DateTime<Utc>,
    pub status: String,
    pub version: i64,
    pub state_name: Option<String>,
    pub payload: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTimerDto {
    pub timer_id: String,
    pub run_id: String,
    pub shard_id: i64,
    pub fire_at: DateTime<Utc>,
    pub status: String,
    pub version: i64,
    pub state_name: Option<String>,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTimerDto {
    pub fire_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub version: Option<i64>,
    pub payload: Option<Option<Value>>,
}
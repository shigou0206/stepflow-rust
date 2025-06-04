use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Timer {
    pub timer_id: String,
    pub run_id: String,
    pub shard_id: i64,
    pub fire_at: NaiveDateTime,
    pub status: String,
    pub version: i64,
    pub state_name: Option<String>,
    pub payload: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateTimer {
    pub fire_at: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub version: Option<i64>,
    pub state_name: Option<Option<String>>,
    pub payload: Option<Option<String>>,
}

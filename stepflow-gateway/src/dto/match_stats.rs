use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct QueueTaskPreview {
    pub run_id: String,
    pub state_name: String,
    pub priority: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct MemoryStats {
    pub total: usize,
    pub peek: Option<QueueTaskPreview>,
}

#[derive(Serialize)]
pub struct DbStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
    pub retrying: usize,
}

#[derive(Serialize)]
pub struct MatchStatsResponse {
    pub memory: MemoryStats,
    pub db: DbStats,
}
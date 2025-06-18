-- Add migration script here
CREATE TABLE activity_tasks (
    task_token TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    shard_id INTEGER NOT NULL,
    seq INTEGER NOT NULL,
    activity_type TEXT NOT NULL,
    state_name TEXT,
    input TEXT,
    result TEXT,
    status TEXT NOT NULL,
    error TEXT,
    error_details TEXT,
    attempt INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    heartbeat_at DATETIME,
    scheduled_at DATETIME NOT NULL,
    started_at DATETIME,
    completed_at DATETIME,
    timeout_seconds INTEGER,
    retry_policy TEXT,
    version INTEGER NOT NULL
);

-- 索引推荐
CREATE INDEX idx_activity_run_seq ON activity_tasks (run_id, seq);
CREATE INDEX idx_activity_status ON activity_tasks (status);
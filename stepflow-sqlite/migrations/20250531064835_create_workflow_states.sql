-- Add migration script here
CREATE TABLE workflow_states (
    state_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    shard_id INTEGER NOT NULL,
    state_name TEXT NOT NULL,
    state_type TEXT NOT NULL,
    status TEXT NOT NULL,
    input TEXT,
    output TEXT,
    error TEXT,
    error_details TEXT,
    started_at DATETIME,
    completed_at DATETIME,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    version INTEGER NOT NULL
);

-- 索引（推荐）
CREATE INDEX idx_workflow_states_run_id ON workflow_states (run_id);
CREATE INDEX idx_workflow_states_status ON workflow_states (status);
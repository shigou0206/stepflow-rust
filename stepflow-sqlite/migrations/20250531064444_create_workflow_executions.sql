-- Add migration script here
CREATE TABLE workflow_executions (
    run_id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    shard_id INTEGER NOT NULL,
    template_id TEXT,
    mode TEXT NOT NULL,
    current_state_name TEXT,
    status TEXT NOT NULL,
    workflow_type TEXT NOT NULL,
    input TEXT,
    input_version INTEGER NOT NULL,
    result TEXT,
    result_version INTEGER NOT NULL,
    start_time DATETIME NOT NULL,
    close_time DATETIME,
    current_event_id INTEGER NOT NULL,
    memo TEXT,
    search_attrs TEXT,
    context_snapshot TEXT,
    version INTEGER NOT NULL
);

-- 推荐索引
CREATE INDEX idx_workflow_executions_shard_status ON workflow_executions (shard_id, status);
-- Add migration script here
CREATE TABLE workflow_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    shard_id INTEGER NOT NULL,
    event_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    state_id TEXT,
    state_type TEXT,
    trace_id TEXT,
    parent_event_id INTEGER,
    context_version INTEGER,
    attributes TEXT,
    attr_version INTEGER NOT NULL,
    timestamp DATETIME NOT NULL,
    archived BOOLEAN NOT NULL DEFAULT FALSE
);

-- 推荐索引 (便于高效检索)
CREATE INDEX idx_wf_events_run_event ON workflow_events(run_id, event_id);
CREATE INDEX idx_wf_events_shard_run_event ON workflow_events(shard_id, run_id, event_id);
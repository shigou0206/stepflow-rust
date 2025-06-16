CREATE TABLE workflow_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    dsl_definition TEXT NOT NULL,
    version INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
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
CREATE INDEX idx_workflow_executions_shard_status ON workflow_executions (shard_id, status);
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
CREATE TABLE sqlite_sequence(name,seq);
CREATE INDEX idx_wf_events_run_event ON workflow_events(run_id, event_id);
CREATE INDEX idx_wf_events_shard_run_event ON workflow_events(shard_id, run_id, event_id);
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
CREATE INDEX idx_workflow_states_run_id ON workflow_states (run_id);
CREATE INDEX idx_workflow_states_status ON workflow_states (status);
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
CREATE INDEX idx_activity_run_seq ON activity_tasks (run_id, seq);
CREATE INDEX idx_activity_status ON activity_tasks (status);
CREATE TABLE timers (
    timer_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    shard_id INTEGER NOT NULL,
    fire_at DATETIME NOT NULL,
    status TEXT NOT NULL,
    version INTEGER NOT NULL,
    state_name TEXT NOT NULL
, payload TEXT, created_at DATETIME NOT NULL DEFAULT '1970-01-01 00:00:00', updated_at DATETIME NOT NULL DEFAULT '1970-01-01 00:00:00');
CREATE INDEX idx_timers_run ON timers(run_id, fire_at);
CREATE TABLE workflow_visibility (
    run_id TEXT PRIMARY KEY,
    workflow_id TEXT,
    workflow_type TEXT,
    start_time DATETIME,
    close_time DATETIME,
    status TEXT,
    memo TEXT,
    search_attrs TEXT,
    version INTEGER NOT NULL
);
CREATE INDEX idx_visibility_status ON workflow_visibility(status);
CREATE INDEX idx_visibility_workflow_id ON workflow_visibility(workflow_id);
CREATE TABLE queue_tasks (
    task_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    state_name TEXT NOT NULL,
    task_payload TEXT,
    
    status TEXT NOT NULL DEFAULT 'queued',
    
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,

    error_message TEXT,
    last_error_at DATETIME,
    next_retry_at DATETIME,

    queued_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processing_at DATETIME,
    completed_at DATETIME,
    failed_at DATETIME,
    
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
, resource TEXT NOT NULL DEFAULT '', priority INTEGER, timeout_seconds INTEGER);
CREATE INDEX idx_queue_tasks_status ON queue_tasks(status);
CREATE INDEX idx_queue_tasks_run_id ON queue_tasks(run_id);
CREATE INDEX idx_queue_tasks_next_retry_at ON queue_tasks(next_retry_at);
CREATE INDEX idx_queue_tasks_updated_at ON queue_tasks(updated_at);

-- migrations/20240608_init.sql

-- === Core Tables ===

-- 1. workflow_templates
CREATE TABLE workflow_templates (
    template_id VARCHAR(64) PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    dsl_definition TEXT NOT NULL,
    version INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- 2. workflow_executions
CREATE TABLE workflow_executions (
    run_id VARCHAR(64) PRIMARY KEY,
    workflow_id VARCHAR(64),
    shard_id INTEGER NOT NULL,
    template_id VARCHAR(64),
    mode TEXT NOT NULL,
    current_state_name TEXT,
    status TEXT NOT NULL,
    workflow_type TEXT NOT NULL,
    input TEXT,
    input_version INTEGER NOT NULL,
    result TEXT,
    result_version INTEGER NOT NULL,
    start_time TIMESTAMP NOT NULL,
    close_time TIMESTAMP,
    current_event_id INTEGER NOT NULL,
    memo TEXT,
    search_attrs TEXT,
    context_snapshot TEXT,
    version INTEGER NOT NULL
);
CREATE INDEX idx_workflow_executions_shard_status ON workflow_executions (shard_id, status);

-- 3. queue_tasks
CREATE TABLE queue_tasks (
    task_id VARCHAR(64) PRIMARY KEY,
    run_id VARCHAR(64) NOT NULL,
    state_name TEXT NOT NULL,
    task_payload TEXT,
    resource TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    priority INTEGER,
    timeout_seconds INTEGER,
    error_message TEXT,
    last_error_at TIMESTAMP,
    next_retry_at TIMESTAMP,
    queued_at TIMESTAMP NOT NULL DEFAULT NOW(),
    processing_at TIMESTAMP,
    completed_at TIMESTAMP,
    failed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_queue_tasks_status ON queue_tasks(status);
CREATE INDEX idx_queue_tasks_run_id ON queue_tasks(run_id);
CREATE INDEX idx_queue_tasks_updated_at ON queue_tasks(updated_at);
CREATE INDEX idx_queue_tasks_next_retry_at ON queue_tasks(next_retry_at);

-- 4. workflow_visibility
CREATE TABLE workflow_visibility (
    run_id VARCHAR(64) PRIMARY KEY,
    workflow_id VARCHAR(64),
    workflow_type TEXT,
    start_time TIMESTAMP,
    close_time TIMESTAMP,
    status TEXT,
    memo TEXT,
    search_attrs TEXT,
    version INTEGER NOT NULL
);
CREATE INDEX idx_visibility_status ON workflow_visibility(status);
CREATE INDEX idx_visibility_workflow_id ON workflow_visibility(workflow_id);

-- 5. timers
CREATE TABLE timers (
    timer_id VARCHAR(64) PRIMARY KEY,
    run_id VARCHAR(64) NOT NULL,
    shard_id INTEGER NOT NULL,
    fire_at TIMESTAMP NOT NULL,
    payload TEXT,
    status TEXT NOT NULL,
    version INTEGER NOT NULL,
    state_name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_timers_run ON timers(run_id, fire_at);

-- 6. activity_tasks
CREATE TABLE activity_tasks (
    task_token VARCHAR(128) PRIMARY KEY,
    run_id VARCHAR(64) NOT NULL,
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
    heartbeat_at TIMESTAMP,
    scheduled_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    timeout_seconds INTEGER,
    retry_policy TEXT,
    version INTEGER NOT NULL
);
CREATE INDEX idx_activity_run_seq ON activity_tasks (run_id, seq);
CREATE INDEX idx_activity_status ON activity_tasks (status);

-- 7. workflow_states
CREATE TABLE workflow_states (
    state_id VARCHAR(64) PRIMARY KEY,
    run_id VARCHAR(64) NOT NULL,
    shard_id INTEGER NOT NULL,
    state_name TEXT NOT NULL,
    state_type TEXT NOT NULL,
    status TEXT NOT NULL,
    input TEXT,
    output TEXT,
    error TEXT,
    error_details TEXT,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version INTEGER NOT NULL
);
CREATE INDEX idx_workflow_states_run_id ON workflow_states(run_id);
CREATE INDEX idx_workflow_states_status ON workflow_states(status);

-- 8. workflow_events
CREATE TABLE workflow_events (
    id SERIAL PRIMARY KEY,
    run_id VARCHAR(64) NOT NULL,
    shard_id INTEGER NOT NULL,
    event_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    state_id VARCHAR(64),
    state_type TEXT,
    trace_id TEXT,
    parent_event_id INTEGER,
    context_version INTEGER,
    attributes TEXT,
    attr_version INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    archived BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX idx_wf_events_run_event ON workflow_events(run_id, event_id);
CREATE INDEX idx_wf_events_shard_run_event ON workflow_events(shard_id, run_id, event_id);

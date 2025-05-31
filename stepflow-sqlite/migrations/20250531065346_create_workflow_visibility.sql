-- Add migration script here
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

-- 推荐索引（用于快速检索）
CREATE INDEX idx_visibility_status ON workflow_visibility(status);
CREATE INDEX idx_visibility_workflow_id ON workflow_visibility(workflow_id);
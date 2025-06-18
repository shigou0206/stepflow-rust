-- Add migration script here
-- migration SQL 文件 (e.g., migrations/20240601120000_update_queue_tasks_schema.sql)

-- 删除旧表 (如果存在)
DROP TABLE IF EXISTS queue_tasks;

-- 创建优化后的 queue_tasks 表结构
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
);

-- 创建索引
CREATE INDEX idx_queue_tasks_status ON queue_tasks(status);
CREATE INDEX idx_queue_tasks_run_id ON queue_tasks(run_id);
CREATE INDEX idx_queue_tasks_next_retry_at ON queue_tasks(next_retry_at);
CREATE INDEX idx_queue_tasks_updated_at ON queue_tasks(updated_at);
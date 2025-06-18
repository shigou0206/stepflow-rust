-- Add migration script here
-- Create queue_tasks table
CREATE TABLE queue_tasks (
    task_id TEXT PRIMARY KEY,                 -- 任务唯一ID
    run_id TEXT NOT NULL,                     -- 对应 workflow_execution 的 run_id
    state_name TEXT NOT NULL,                 -- 工作流状态名或任务类型
    task_payload TEXT,                        -- JSON格式的任务数据
    status TEXT NOT NULL,                     -- 任务状态：pending, processing, succeeded, failed
    attempts INTEGER NOT NULL DEFAULT 0,      -- 当前重试次数
    max_attempts INTEGER NOT NULL DEFAULT 3,  -- 最大允许的重试次数
    error_message TEXT,                       -- 最近一次错误详情
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP  -- 最近更新时间
);

-- 创建索引以提高性能
CREATE INDEX idx_queue_tasks_status ON queue_tasks(status);
CREATE INDEX idx_queue_tasks_run_id ON queue_tasks(run_id);
CREATE INDEX idx_queue_tasks_updated_at ON queue_tasks(updated_at);
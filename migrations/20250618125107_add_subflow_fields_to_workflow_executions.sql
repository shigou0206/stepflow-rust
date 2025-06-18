-- Add migration script here
-- 添加子流程归属字段
ALTER TABLE workflow_executions
ADD COLUMN parent_run_id TEXT;

ALTER TABLE workflow_executions
ADD COLUMN parent_state_name TEXT;

-- 添加子流程内联 DSL 字段（JSON 格式）
ALTER TABLE workflow_executions
ADD COLUMN dsl_definition TEXT;

-- 创建子流程快速聚合查询索引
CREATE INDEX idx_workflow_executions_parent
ON workflow_executions (parent_run_id, parent_state_name);
-- Add priority and timeout_seconds to queue_tasks table

ALTER TABLE queue_tasks
    ADD COLUMN priority INTEGER;

ALTER TABLE queue_tasks
    ADD COLUMN timeout_seconds INTEGER;
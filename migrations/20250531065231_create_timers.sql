-- Add migration script here
CREATE TABLE timers (
    timer_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    shard_id INTEGER NOT NULL,
    fire_at DATETIME NOT NULL,
    status TEXT NOT NULL,
    version INTEGER NOT NULL,
    state_name TEXT NOT NULL
);

-- 推荐索引
CREATE INDEX idx_timers_run ON timers(run_id, fire_at);
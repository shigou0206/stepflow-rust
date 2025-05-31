// tests/wait_tests.rs
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::SqlitePool;
use stepflow_dsl::WorkflowDSL;
use stepflow_storage::PersistenceManagerImpl;
use std::sync::Arc;
use stepflow_engine::{
    engine::{
        memory_stub::{MemoryQueue, MemoryStore},
        WorkflowEngine, WorkflowMode,
    },
};
use stepflow_hook::{EngineEventDispatcher, impls::log_hook::LogHook};
use chrono::{Utc, Duration};

static TEST_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    SqlitePool::connect_lazy("sqlite::memory:").unwrap()
});

static TEST_PERSISTENCE: Lazy<Arc<PersistenceManagerImpl>> = Lazy::new(|| {
    Arc::new(PersistenceManagerImpl::new(TEST_POOL.clone()))
});

#[tokio::test]
async fn wait_seconds_inline() {
    let dsl: WorkflowDSL = serde_json::from_str(r#"
    {
      "StartAt": "Wait1",
      "States": {
        "Wait1": { "Type": "Wait", "Seconds": 1, "Next": "Done" },
        "Done": { "Type": "Pass", "Result": {"ok": true}, "End": true }
      }
    }
    "#).unwrap();
    let engine = WorkflowEngine::new(
        "r".into(), dsl, json!({}),
        WorkflowMode::Inline, MemoryStore::new(TEST_PERSISTENCE.clone()), MemoryQueue::new(), TEST_POOL.clone(),
        Arc::new(EngineEventDispatcher::new(vec![LogHook::new()]))
    );
    let out = engine.run_inline().await.unwrap();
    assert_eq!(out["ok"], true);
}

#[tokio::test]
async fn wait_timestamp_inline() {
    let future = (Utc::now() + Duration::seconds(1)).to_rfc3339();
    let dsl = format!(r#"
    {{
      "StartAt":"Wait1",
      "States": {{
        "Wait1":{{"Type":"Wait","Timestamp":"{}","Next":"Done"}},
        "Done":{{"Type":"Pass","Result":{{"ok":true}},"End":true}}
      }}
    }}
    "#, future);
    let dsl: WorkflowDSL = serde_json::from_str(&dsl).unwrap();
    let engine = WorkflowEngine::new(
        "r".into(), dsl, json!({}),
        WorkflowMode::Inline, MemoryStore::new(TEST_PERSISTENCE.clone()), MemoryQueue::new(), TEST_POOL.clone(),
        Arc::new(EngineEventDispatcher::new(vec![LogHook::new()]))
    );
    let out = engine.run_inline().await.unwrap();
    assert_eq!(out["ok"], true);
}
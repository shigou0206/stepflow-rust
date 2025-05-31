// tests/terminal_tests.rs
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::SqlitePool;
use stepflow_dsl::WorkflowDSL;
use stepflow_engine::{
    engine::{memory_stub::{MemoryQueue, MemoryStore}, WorkflowEngine, WorkflowMode},
};

static TEST_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    SqlitePool::connect_lazy("sqlite::memory:").unwrap()
});

#[tokio::test]
async fn fail_state_inline() {
    let dsl: WorkflowDSL = serde_json::from_str(r#"
    {
      "StartAt":"ErrorState",
      "States": {
        "ErrorState": {
          "Type":"Fail",
          "Error":"BadThings",
          "Cause":"just because"
        }
      }
    }
    "#).unwrap();

    let engine = WorkflowEngine::new(
        "r".into(), dsl, json!({}),
        WorkflowMode::Inline, MemoryStore, MemoryQueue::new(), TEST_POOL.clone()
    );
    let err = engine.run_inline().await.unwrap_err();
    assert!(err.contains("BadThings"));
}

#[tokio::test]
async fn succeed_pass_inline() {
    let dsl: WorkflowDSL = serde_json::from_str(r#"
    {
      "StartAt":"OnlyPass",
      "States": {
        "OnlyPass":{
          "Type":"Pass",
          "Result":{"bye":"world"},
          "End":true
        }
      }
    }
    "#).unwrap();

    let out = WorkflowEngine::new(
        "r".into(), dsl, json!({}),
        WorkflowMode::Inline, MemoryStore, MemoryQueue::new(), TEST_POOL.clone()
    ).run_inline().await.unwrap();
    assert_eq!(out["bye"], "world");
}
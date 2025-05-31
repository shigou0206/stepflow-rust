// stepflow-engine/tests/engine_tests.rs
//! Integration‑level tests covering various state types

use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_dsl::dsl::WorkflowDSL;
use stepflow_storage::PersistenceManagerImpl;
use std::sync::Arc;
use stepflow_engine::engine::{
    WorkflowEngine, WorkflowMode, memory_stub::MemoryQueue, memory_stub::MemoryStore,
};

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

static TEST_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
    futures::executor::block_on(MIGRATOR.run(&pool)).expect("Failed to run migrations");
    pool
});

static TEST_PERSISTENCE: Lazy<Arc<PersistenceManagerImpl>> = Lazy::new(|| {
    Arc::new(PersistenceManagerImpl::new(TEST_POOL.clone()))
});

// -----------------------------------------------------------------------------
// Helper to build engine quickly
// -----------------------------------------------------------------------------
fn build_engine(dsl: WorkflowDSL, mode: WorkflowMode) -> WorkflowEngine<MemoryStore, MemoryQueue> {
    WorkflowEngine::new(
        "run‑test".into(),
        dsl,
        json!({}),
        mode,
        MemoryStore::new(TEST_PERSISTENCE.clone()),
        MemoryQueue::new(),
        TEST_POOL.clone(),
    )
}

// -----------------------------------------------------------------------------
// 1. Wait → Pass → End (Inline)
// -----------------------------------------------------------------------------
#[tokio::test]
async fn wait_then_pass_inline() {
    const DSL: &str = r#"{
        "StartAt": "Wait1",
        "States": {
          "Wait1": { "Type": "Wait", "Seconds": 0, "Next": "Pass1" },
          "Pass1": { "Type": "Pass", "Result": {"ok": true}, "End": true }
        }
    }"#;
    let dsl: WorkflowDSL = serde_json::from_str(DSL).unwrap();
    let engine = build_engine(dsl, WorkflowMode::Inline);
    let out = engine.run_inline().await.unwrap();
    assert!(out["ok"].as_bool().unwrap());
}

// -----------------------------------------------------------------------------
// 2. Choice branching (x == 1)
// -----------------------------------------------------------------------------
#[tokio::test]
async fn choice_branch_inline() {
    const DSL: &str = r#"
    {
        "StartAt": "Choice1",
        "States": {
            "Choice1": {
            "Type": "Choice",
            "Choices": [
                {
                "Condition": {
                    "Variable": "$.x",
                    "Operator": "Equals",
                    "Value": 1
                },
                "Next": "S1"
                }
            ],
            "Default": "Fail1"
            },
            "S1":    { "Type": "Succeed" },
            "Fail1": { "Type": "Fail", "Error": "no", "Cause": "branch" }
        }
    }
    "#;
    let dsl: WorkflowDSL = serde_json::from_str(DSL).unwrap();
    // Set initial context
    let mut engine = build_engine(dsl.clone(), WorkflowMode::Inline);
    engine.context = json!({"x": 1});
    let out = engine.run_inline().await.unwrap();
    assert_eq!(out, json!({"x": 1}));
}

// -----------------------------------------------------------------------------
// 3. Deferred Task should enqueue and pause
// -----------------------------------------------------------------------------
#[tokio::test]
async fn deferred_task_enqueue() {
    const DSL: &str = r#"{
        "StartAt": "Task1",
        "States": {
          "Task1": {"Type": "Task", "Resource": "echo", "End": true}
        }
    }"#;
    let dsl: WorkflowDSL = serde_json::from_str(DSL).unwrap();
    let mut engine = build_engine(dsl, WorkflowMode::Deferred);

    let outcome = engine.advance_once().await.unwrap();
    assert!(!outcome.should_continue); // paused

    // Queue should contain one entry
    let len = {
        let guard = engine.queue.0.lock().await;
        guard.len()
    };
    assert_eq!(len, 1);
}

// -----------------------------------------------------------------------------
// 4. Fail state terminates with error
// -----------------------------------------------------------------------------
#[tokio::test]
async fn fail_state_inline() {
    const DSL: &str = r#"{
        "StartAt": "Fail1",
        "States": {
          "Fail1": {"Type": "Fail", "Error": "boom", "Cause": "testing"}
        }
    }"#;
    let dsl: WorkflowDSL = serde_json::from_str(DSL).unwrap();
    let engine = build_engine(dsl, WorkflowMode::Inline);

    // 由于 Fail 节点会直接报错，这里应当 unwrap_err() 并检查错误里包含 "boom"
    let err = engine.run_inline().await.unwrap_err();
    assert!(err.contains("boom"));
}

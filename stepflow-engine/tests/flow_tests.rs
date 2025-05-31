// tests/flow_tests.rs
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::SqlitePool;
use stepflow_dsl::WorkflowDSL;
use stepflow_storage::PersistenceManagerImpl;
use std::sync::Arc;
use stepflow_engine::{
    engine::{memory_stub::{MemoryStore, MemoryQueue}, WorkflowEngine, WorkflowMode},
};
use stepflow_hook::{EngineEventDispatcher, impls::log_hook::LogHook};

static TEST_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    SqlitePool::connect_lazy("sqlite::memory:").unwrap()
});

static TEST_PERSISTENCE: Lazy<Arc<PersistenceManagerImpl>> = Lazy::new(|| {
    Arc::new(PersistenceManagerImpl::new(TEST_POOL.clone()))
});

#[tokio::test]
async fn full_flow_inline() {
    // 1️⃣ 这个 DSL 原本用了 "NumericGreaterThan"，改成通用 Operator/Value 形式
    let dsl: WorkflowDSL = serde_json::from_str(r#"
    {
      "StartAt": "T1",
      "States": {
        "T1": {
          "Type": "Task",
          "Resource": "echo",
          "Next": "P1"
        },
        "P1": {
          "Type": "Pass",
          "Result": { "x": 100 },
          "Next": "C1"
        },
        "C1": {
          "Type": "Choice",
          "Choices": [
            {
              "Condition": {
                "Variable": "$.x",
                "Operator": "GreaterThan",
                "Value": 50
              },
              "Next": "Good"
            }
          ],
          "Default": "Bad"
        },
        "Good": {
          "Type": "Succeed"
        },
        "Bad": {
          "Type": "Fail",
          "Error": "TooSmall",
          "Cause": ""
        }
      }
    }
    "#).unwrap();

    // 2️⃣ 指定初始上下文为 { "foo": 1 }，T1 会把输出打到 _ran
    let out = WorkflowEngine::new(
        "r".into(),
        dsl.clone(),
        json!({ "foo": 1 }),
        WorkflowMode::Inline,
        MemoryStore::new(TEST_PERSISTENCE.clone()),
        MemoryQueue::new(),
        TEST_POOL.clone(),
        Arc::new(EngineEventDispatcher::new(vec![LogHook::new()])),
    )
    .run_inline()
    .await
    .unwrap();

    // 3️⃣ T1 会在 context 上打 _ran = "tool::echo"
    assert_eq!(out["_ran"], "tool::echo");

    // 4️⃣ P1 会把 x 覆盖为 100
    assert_eq!(out["x"], 100);

    // 5️⃣ C1 分支 { x > 50 }，选择 Good → Succeed，最终返回的 context 里依然保留 _ran
    assert_eq!(out["_ran"], "tool::echo");
}
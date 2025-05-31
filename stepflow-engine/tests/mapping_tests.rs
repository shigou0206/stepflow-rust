// stepflow-engine/tests/mapping_tests.rs
//! Verify Input-Mapping + Parameters integration inside Task handler.

use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use stepflow_dsl::WorkflowDSL;
use stepflow_storage::PersistenceManagerImpl;
use std::sync::Arc;
use stepflow_engine::engine::{
    memory_stub::{MemoryQueue, MemoryStore},
    WorkflowEngine,
    WorkflowMode,
};

static TEST_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    SqlitePool::connect_lazy("sqlite::memory:").unwrap()
});

static TEST_PERSISTENCE: Lazy<Arc<PersistenceManagerImpl>> = Lazy::new(|| {
    Arc::new(PersistenceManagerImpl::new(TEST_POOL.clone()))
});

/// DSL: Task1 先把 `$.u` 映射到 `user`，然后把
///      Parameters 映射为 { uid, msg }
const DSL_MAPPING: &str = r#"
{
  "StartAt": "Task1",
  "States": {
    "Task1": {
      "Type": "Task",
      "Resource": "echo",
      "InputMapping": {
        "mappings": [
          { "key": "user", "type": "jsonPath", "source": "$.u" },
          { "key": "uid", "type": "jsonPath", "source": "$.u.id" },
          { "key": "msg", "type": "constant", "value": "hi" }
        ]
      },
      "End": true
    }
  }
}
"#;

#[tokio::test]
async fn task_input_mapping_inline() {
    // 1️⃣ 解析 DSL
    let dsl: WorkflowDSL = serde_json::from_str(DSL_MAPPING).unwrap();

    // 2️⃣ 初始上下文
    let init_ctx = json!({ "u": { "id": 42, "name": "Bob" } });

    // 3️⃣ 构建 Engine（Inline + Memory stub store/queue）
    let engine = WorkflowEngine::new(
        "run-map".into(),
        dsl,
        init_ctx,
        WorkflowMode::Inline,
        MemoryStore::new(TEST_PERSISTENCE.clone()),
        MemoryQueue::new(),
        TEST_POOL.clone(),
    );

    // 4️⃣ 执行
    let out: Value = engine.run_inline().await.unwrap();

    // 5️⃣ 断言
    assert_eq!(out["uid"], 42);
    assert_eq!(out["msg"], "hi");
    assert_eq!(out["_ran"], "tool::echo");
}
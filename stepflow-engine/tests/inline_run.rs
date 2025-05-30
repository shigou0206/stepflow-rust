//! 集成测试：Inline 模式 Task → Pass → 终态

use serde_json::json;
use stepflow_dsl::dsl::WorkflowDSL;
use stepflow_engine::{
    engine::{
        WorkflowEngine,
        WorkflowMode,
        memory_stub::{MemoryStore, MemoryQueue},
    },
};

/// DSL: Task1 执行后进入 Pass1，Pass1 输出 `"msg":"done"` 并终止
const SIMPLE_DSL: &str = r#"
{
  "StartAt": "Task1",
  "States": {
    "Task1": {
      "Type": "Task",
      "Resource": "echo",
      "Next": "Pass1"
    },
    "Pass1": {
      "Type": "Pass",
      "Result": { "msg": "done" },
      "End": true
    }
  }
}
"#;

#[tokio::test]
async fn run_inline_task_pass() {
    // 1. 解析 DSL
    let dsl: WorkflowDSL = serde_json::from_str(SIMPLE_DSL).unwrap();

    // 2. 构建引擎（Inline + 内存 Stub）
    let engine = WorkflowEngine::new(
        "run-1".into(),
        dsl,
        json!({}),            // 初始上下文
        WorkflowMode::Inline,
        MemoryStore,          // 内存 TaskStore
        MemoryQueue::new(),   // 内存 TaskQueue
    );

    // 3. 跑到底
    let out = engine.run_inline().await.unwrap();

    // 4. 断言结果
    assert_eq!(out["msg"], "done");
}
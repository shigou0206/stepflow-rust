// stepflow-engine/tests/choice_tests.rs
//! 验证 Choice State 分支行为（通用 Operator + Value 语法）

use serde_json::json;
use stepflow_dsl::dsl::WorkflowDSL;
use stepflow_engine::{
    engine::{
        memory_stub::{MemoryQueue, MemoryStore},
        WorkflowEngine,
        WorkflowMode,
    },
};

const DSL_CHOICE: &str = r#"
{
  "StartAt": "Decider",
  "States": {
    "Decider": {
      "Type": "Choice",
      "Choices": [
        {
          "Condition": {
            "Variable": "$.x",
            "Operator": "GreaterThan",
            "Value": 10
          },
          "Next": "Big"
        },
        {
          "Condition": {
            "Variable": "$.x",
            "Operator": "LessThanEquals",
            "Value": 10
          },
          "Next": "Small"
        }
      ]
    },
    "Big":   { "Type": "Pass", "Result": { "tag": "big" }, "End": true },
    "Small": { "Type": "Pass", "Result": { "tag": "small" }, "End": true }
  }
}
"#;

#[tokio::test]
async fn choice_branch_inline() {
    let dsl: WorkflowDSL = serde_json::from_str(DSL_CHOICE).unwrap();

    // x > 10 → Big
    let engine = WorkflowEngine::new(
        "run1".into(),
        dsl.clone(),
        json!({"x": 42}),
        WorkflowMode::Inline,
        MemoryStore,
        MemoryQueue::new(),
    );
    let out = engine.run_inline().await.unwrap();
    assert_eq!(out["tag"], "big");

    // x <= 10 → Small
    let engine = WorkflowEngine::new(
        "run2".into(),
        dsl,
        json!({"x": 5}),
        WorkflowMode::Inline,
        MemoryStore,
        MemoryQueue::new(),
    );
    let out = engine.run_inline().await.unwrap();
    assert_eq!(out["tag"], "small");
}
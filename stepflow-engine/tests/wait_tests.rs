// tests/wait_tests.rs
use serde_json::json;
use stepflow_dsl::WorkflowDSL;
use stepflow_engine::{
    engine::{
        memory_stub::{MemoryQueue, MemoryStore},
        WorkflowEngine, WorkflowMode,
    },
};
use chrono::{Utc, Duration};

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
        WorkflowMode::Inline, MemoryStore, MemoryQueue::new()
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
        WorkflowMode::Inline, MemoryStore, MemoryQueue::new()
    );
    let out = engine.run_inline().await.unwrap();
    assert_eq!(out["ok"], true);
}
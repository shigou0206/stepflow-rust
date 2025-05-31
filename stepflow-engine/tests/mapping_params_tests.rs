// tests/mapping_params_tests.rs

use serde_json::{json, Value};
use stepflow_dsl::WorkflowDSL;
use stepflow_engine::{
    engine::{memory_stub::{MemoryStore, MemoryQueue}, WorkflowEngine, WorkflowMode},
};

#[tokio::test]
async fn input_and_output_mapping() {
    // 1️⃣ DSL 里：
    //    - 把 x 映射到 a
    //    - 同时把常量 "foo" 映射到 b
    //    - 业务执行完毕后，只用 OutputMapping 把 _ran 抽取到 c
    //
    // 由于当前引擎实现会将 OutputMapping 的结果合并到“原始上下文”上，
    // 最终保留 x 和 c，而 a、b 会在业务环节生成后被丢弃。
    let dsl: WorkflowDSL = serde_json::from_str(r#"
    {
      "StartAt": "T1",
      "States": {
        "T1": {
          "Type": "Task",
          "Resource": "echo",

          "InputMapping": {
            "mappings": [
              { "key": "a", "type": "jsonPath", "source": "$.x" },
              { "key": "b", "type": "constant", "value": "foo" }
            ]
          },

          "OutputMapping": {
            "mappings": [
              { "key": "c", "type": "jsonPath", "source": "$._ran" }
            ]
          },

          "End": true
        }
      }
    }
    "#).unwrap();

    // 2️⃣ 初始上下文里只包含 x = 123
    let init_ctx = json!({ "x": 123 });

    // 3️⃣ 构造并执行引擎
    let out: Value = WorkflowEngine::new(
        "run-mapping".into(),
        dsl,
        init_ctx.clone(),
        WorkflowMode::Inline,
        MemoryStore,
        MemoryQueue::new()
    )
    .run_inline()
    .await
    .unwrap();

    // 4️⃣ 断言：
    //    - 原始上下文的 x 应当保留
    assert_eq!(out["x"], 123);

    //    - 业务逻辑（echo）会把 "_ran": "tool::echo" 写到输出里，
    //      OutputMapping 将它抽取到 c，所以最终 out["c"] == "tool::echo"
    assert_eq!(out["c"], "tool::echo");

    // 注意：因为当前版本的引擎把 OutputMapping 合并到“原始上下文”上，
    // 所以这里 a、b 虽然经过 InputMapping 和业务逻辑产生，但并不会出现在最终结果里。
    // 如果想要保留 a、b，需要改动引擎的合并逻辑（将 OutputMapping 合并在业务逻辑产生的上下文上）。
}
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

use stepflow_common::config::StepflowConfig;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::worker::{TaskDetails, TaskStatus};
use stepflow_eventbus::global::dispatch_event;
use stepflow_tool::core::registry::ToolRegistry;
use tracing::{info, error};

pub async fn execute_task(
    _client: &Client,          
    _config: &StepflowConfig, 
    registry: &ToolRegistry,
    task: TaskDetails,
) -> Result<()> {
    let start = Instant::now();

    // 先 clone 关键字段，防止多次使用移动值
    let run_id = task.run_id.clone();
    let state_name = task.state_name.clone();

    info!(
        %run_id,
        state = %state_name,
        tool = %task.tool_type,
        "⚙️ Starting task execution",
    );

    let exec_result = registry
        .execute(&task.tool_type, task.parameters.clone())
        .await;

    let (status, output) = match exec_result {
        Ok(ok) => {
            info!(%run_id, "✅ Task succeeded");
            (TaskStatus::SUCCEEDED, ok.output)
        }
        Err(e) => {
            error!(%run_id, err = %e, "❌ Task failed");
            (TaskStatus::FAILED, json!({ "error": e.to_string() }))
        }
    };

    // 🔔 分发事件：TaskFinished
    info!(%run_id, state = %state_name, "📤 Dispatching TaskFinished event");

    dispatch_event(EngineEvent::TaskFinished {
        run_id,
        state_name,
        output,
    }).await;

    info!(
        run_id = %task.run_id,
        elapsed = %start.elapsed().as_millis(),
        "🏁 Task execution + dispatch complete"
    );

    Ok(())
}
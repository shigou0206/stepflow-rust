use anyhow::Result;
use stepflow_core::service::ExecutionSvc;
use stepflow_core::service::ExecutionService;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::execution::ExecStart;
use stepflow_eventbus::core::bus::EventBus;
use std::sync::Arc;

/// 监听 SubflowReady 事件并自动启动子工作流
pub async fn start_subflow_worker(
    exec_service: Arc<ExecutionSvc>,
    event_bus: Arc<dyn EventBus>,
) -> Result<()> {
    let mut rx = event_bus.subscribe();

    while let Ok(envelope) = rx.recv().await {
        if let EngineEvent::SubflowReady {
            run_id,
            parent_run_id,
            state_name,
        } = envelope.payload
        {
            tracing::info!(
                "🚀 Starting subflow: {} (parent={}, state={})",
                run_id, parent_run_id, state_name
            );

            let parent = match exec_service.get(&parent_run_id).await {
                Ok(parent) => parent,
                Err(e) => {
                    tracing::error!("❌ Failed to fetch parent {}: {e:#}", parent_run_id);
                    continue;
                }
            };

            let req = ExecStart {
                template_id: None,
                mode: parent.mode.clone(),
                init_ctx: parent.result.clone(),
                parent_run_id: Some(parent_run_id.clone()),
                parent_state_name: Some(state_name.clone()),
                dsl: None,
            };

            if let Err(e) = exec_service.start(req).await {
                tracing::error!("❌ Failed to start subflow {}: {e:#}", run_id);
            } else {
                tracing::info!("✅ Subflow {} started", run_id);
            }
        }
    }

    Ok(())
}
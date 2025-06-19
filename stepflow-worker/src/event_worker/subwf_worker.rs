use anyhow::Result;
use stepflow_core::app_state::AppState;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::execution::ExecStart;
use stepflow_engine::engine::WorkflowMode;
use stepflow_eventbus::core::bus::EventBus;
use std::sync::Arc;

pub async fn start_subflow_worker(
    app_state: Arc<AppState>,
    bus: Arc<dyn EventBus>,
) -> Result<()> {
    let mut rx = bus.subscribe();

    while let Ok(envelope) = rx.recv().await {
        if let EngineEvent::SubflowReady {
            run_id,
            parent_run_id,
            state_name,
        } = envelope.payload
        {
            let persist = app_state.persist.clone();
            let exec_opt = persist.get_execution(&run_id).await?;

            let Some(exec) = exec_opt else {
                tracing::warn!("⚠️ Subflow not found in DB: {}", run_id);
                continue;
            };

            let dsl = exec
                .dsl_definition
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing dsl_definition in subflow"))?;

            let input = exec.input.clone().unwrap_or_default();
            let mode_str = exec.mode.clone();

            let mode = match mode_str.as_str() {
                "INLINE" => WorkflowMode::Inline,
                "DEFERRED" => WorkflowMode::Deferred,
                other => {
                    tracing::warn!("⚠️ Invalid mode '{}', defaulting to DEFERRED", other);
                    WorkflowMode::Deferred
                }
            };

            tracing::info!(
                "🧷 Launching subflow: {} (parent: {}, state: {})",
                run_id,
                parent_run_id,
                state_name
            );

            let exec_svc = app_state.exec_svc.clone();
            let run_id_clone = run_id.clone();
            let parent_run_id = parent_run_id.clone();
            let state_name = state_name.clone();

            // ✅ 异步启动子流程
            tokio::spawn(async move {
                if let Err(e) = exec_svc
                    .start(ExecStart {
                        run_id: Some(run_id_clone),
                        dsl: Some(dsl),
                        init_ctx: Some(input),
                        mode: mode_str,
                        template_id: None,
                        parent_run_id: Some(parent_run_id),
                        parent_state_name: Some(state_name),
                    })
                    .await
                {
                    tracing::error!("❌ Failed to start subflow {}: {e:#}", run_id);
                }
            });
        }
    }

    Ok(())
}
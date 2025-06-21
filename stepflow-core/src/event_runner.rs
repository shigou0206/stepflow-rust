use crate::app_state::AppState;
use stepflow_common::config::{StepflowConfig, StepflowExecMode};
use stepflow_dto::dto::engine_event::EngineEvent;
use tracing::debug;

use crate::event_handlers::*;

pub fn start_event_runner(app: AppState) {
    let mut rx = app.subscribe_events();

    tokio::spawn(async move {
        while let Ok(envelope) = rx.recv().await {
            match &envelope.payload {
                EngineEvent::TaskFinished { run_id, state_name, output } => {
                    handle_task_finished(&app, run_id, state_name, output).await;
                }

                EngineEvent::SubflowFinished { parent_run_id, child_run_id, state_name, result } => {
                    handle_subflow_finished(&app, parent_run_id, child_run_id, state_name, result).await;
                }

                EngineEvent::SubflowReady { run_id, parent_run_id, state_name, dsl, init_ctx } => {
                    handle_subflow_ready(&app, run_id, parent_run_id, state_name, dsl, init_ctx).await;
                }

                _ => {} // 忽略
            }
        }

        debug!("🛑 Event runner exiting");
    });
}

pub fn maybe_start_event_runner(config: &StepflowConfig, app_state: &AppState) {
    if config.exec_mode == StepflowExecMode::EventDriven {
        tracing::info!("🔔 Starting engine event runner...");
        start_event_runner(app_state.clone());
    }
}

pub fn spawn_event_logger(app_state: &AppState) {
    let mut bus_rx = app_state.subscribe_events();
    tokio::spawn(async move {
        while let Ok(envelope) = bus_rx.recv().await {
            tracing::debug!(?envelope, "🔔 Got EventEnvelope from EventBus");
        }
        tracing::warn!("⚠️ EventBus subscription closed");
    });
}
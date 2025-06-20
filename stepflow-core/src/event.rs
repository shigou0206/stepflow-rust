use tracing::{debug, error, info};
use crate::app_state::AppState;
use stepflow_common::config::{StepflowConfig, StepflowExecMode};
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::execution::ExecStart;
use stepflow_dto::dto::signal::ExecutionSignal;

/// 启动事件驱动执行器（含 TaskFinished / SubflowFinished / SubflowReady）
pub fn start_event_runner(app: AppState) {
    let mut rx = app.subscribe_events();

    tokio::spawn(async move {
        while let Ok(envelope) = rx.recv().await {
            match &envelope.payload {
                // ✅ TaskFinished → TaskCompleted signal
                EngineEvent::TaskFinished {
                    run_id,
                    state_name,
                    output,
                } => {
                    info!(%run_id, %state_name, "📩 Received TaskFinished");

                    let mut engines = app.engines.lock().await;
                    let engine = match engines.get_mut(run_id) {
                        Some(e) => e,
                        None => {
                            debug!(%run_id, "⚠️ No active engine found");
                            continue;
                        }
                    };

                    let signal = ExecutionSignal::TaskCompleted {
                        run_id: run_id.clone(),
                        state_name: state_name.clone(),
                        output: output.clone(),
                    };

                    if let Some(tx) = engine.get_signal_sender() {
                        if let Err(e) = tx.send(signal) {
                            error!(%run_id, ?e, "❌ Failed to send signal");
                            continue;
                        }
                    } else {
                        error!(%run_id, "❌ Missing signal sender");
                        continue;
                    }

                    info!(%run_id, %state_name, "✅ Signal sent, advancing workflow...");
                    if let Err(e) = engine.handle_next_signal().await {
                        error!(%run_id, ?e, "❌ handle_next_signal failed");
                        continue;
                    }
                    if let Err(e) = engine.advance_until_blocked().await {
                        error!(%run_id, ?e, "❌ advance_until_blocked failed");
                        continue;
                    }
                    info!(%run_id, "✅ advance_until_blocked complete");

                    if engine.finished {
                        engines.remove(run_id);
                        info!(%run_id, "🏁 workflow finished, engine removed");
                    }
                }

                // ✅ SubflowFinished → 发送 SubflowFinished signal
                EngineEvent::SubflowFinished {
                    parent_run_id,
                    child_run_id,
                    state_name,
                    result,
                } => {
                    info!(%parent_run_id, %child_run_id, %state_name, "📩 Received SubflowFinished");

                    let mut engines = app.engines.lock().await;
                    let engine = match engines.get_mut(parent_run_id) {
                        Some(e) => e,
                        None => {
                            debug!(%parent_run_id, "⚠️ No active parent engine found");
                            continue;
                        }
                    };

                    let signal = ExecutionSignal::SubflowFinished {
                        parent_run_id: parent_run_id.clone(),
                        child_run_id: child_run_id.clone(),
                        state_name: state_name.clone(),
                        result: result.clone(),
                    };

                    if let Some(tx) = engine.get_signal_sender() {
                        if let Err(e) = tx.send(signal) {
                            error!(%parent_run_id, ?e, "❌ Failed to send subflow signal");
                            continue;
                        }
                    } else {
                        error!(%parent_run_id, "❌ Missing signal sender");
                        continue;
                    }

                    info!(%parent_run_id, %state_name, "✅ Subflow signal sent");

                    if let Err(e) = engine.handle_next_signal().await {
                        error!(%parent_run_id, ?e, "❌ handle_next_signal failed");
                        continue;
                    }
                    if let Err(e) = engine.advance_until_blocked().await {
                        error!(%parent_run_id, ?e, "❌ advance_until_blocked failed");
                        continue;
                    }

                    if engine.finished {
                        engines.remove(parent_run_id);
                        info!(%parent_run_id, "🏁 parent workflow finished, engine removed");
                    }
                }

                // ✅ SubflowReady → 启动子工作流
                // EngineEvent::SubflowReady {
                //     run_id,
                //     parent_run_id,
                //     state_name,
                //     dsl,
                //     init_ctx,
                // } => {
                //     info!(
                //         %run_id, %parent_run_id, %state_name,
                //         "📦 SubflowReady received, starting subflow"
                //     );

                //     let exec_svc = app.execution_svc.clone();
                //     let req = ExecStart {
                //         run_id: Some(run_id.clone()),
                //         template_id: None,
                //         init_ctx: Some(init_ctx.clone()),
                //         parent_run_id: Some(parent_run_id.clone()),
                //         parent_state_name: Some(state_name.clone()),
                //         dsl: Some(dsl.clone()),
                //     };

                //     if let Err(e) = exec_svc.start(req).await {
                //         error!(%run_id, "❌ Failed to start subflow: {e:#}");
                //     } else {
                //         info!(%run_id, "✅ Subflow started");
                //     }
                // }

                _ => {} // 忽略其他事件
            }
        }

        debug!("🛑 Event runner exiting");
    });
}

/// 如果是事件驱动模式，则启动引擎监听器
pub fn maybe_start_event_runner(config: &StepflowConfig, app_state: &AppState) {
    if config.exec_mode == StepflowExecMode::EventDriven {
        tracing::info!("🔔 Starting engine event runner...");
        start_event_runner(app_state.clone());
    }
}

/// 启动事件日志打印监听器（可选调试用）
pub fn spawn_event_logger(app_state: &AppState) {
    let mut bus_rx = app_state.subscribe_events();
    tokio::spawn(async move {
        while let Ok(envelope) = bus_rx.recv().await {
            tracing::debug!(?envelope, "🔔 Got EventEnvelope from EventBus");
        }
        tracing::warn!("⚠️ EventBus subscription closed");
    });
}
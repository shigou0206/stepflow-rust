use tracing::{debug, error, info};

use stepflow_core::app_state::AppState;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::signal::ExecutionSignal;

/// 启动事件驱动的执行器 —— 监听 EventBus 上的 TaskFinished 并推进流程
pub fn start_event_runner(app: AppState) {
    let mut rx = app.subscribe_events();

    tokio::spawn(async move {
        while let Ok(envelope) = rx.recv().await {
            match &envelope.payload {
                EngineEvent::TaskFinished {
                    run_id,
                    state_name,
                    output,
                } => {
                    info!(run_id, state_name, "📩 received TaskFinished event");

                    let mut engines = app.engines.lock().await;
                    let engine = match engines.get_mut(run_id) {
                        Some(e) => e,
                        None => {
                            debug!(run_id, "⚠️ no active engine");
                            continue;
                        }
                    };

                    // 推送信号
                    let signal = ExecutionSignal::TaskCompleted {
                        run_id: run_id.clone(),
                        state_name: state_name.clone(),
                        output: output.clone(),
                    };

                    if let Some(tx) = engine.get_signal_sender() {
                        if let Err(e) = tx.send(signal) {
                            error!(run_id, ?e, "❌ failed to send signal to engine");
                            continue;
                        }
                    } else {
                        error!(run_id, "❌ engine signal sender missing");
                        continue;
                    }

                    // 尝试推进流程
                    if let Err(e) = engine.handle_next_signal().await {
                        error!(run_id, ?e, "❌ handle_next_signal failed");
                        continue;
                    }

                    if let Err(e) = engine.advance_until_blocked().await {
                        error!(run_id, ?e, "❌ advance_until_blocked failed");
                        continue;
                    }

                    if engine.finished {
                        engines.remove(run_id);
                        info!(run_id, "🏁 workflow finished, engine removed");
                    }
                }
                _ => {} // 忽略其他事件
            }
        }
        debug!("🛑 event runner exiting");
    });
}

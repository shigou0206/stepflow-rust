use tracing::{debug, error, info};
use crate::app_state::AppState;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::signal::ExecutionSignal;

/// 启动事件驱动执行器
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

                    match engine.get_signal_sender() {
                        Some(tx) => {
                            if let Err(e) = tx.send(signal) {
                                error!(%run_id, ?e, "❌ Failed to send signal");
                                continue;
                            }
                        }
                        None => {
                            error!(%run_id, "❌ Missing signal sender");
                            continue;
                        }
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
                _ => {} // ignore other events
            }
        }

        debug!("🛑 Event runner exiting");
    });
}
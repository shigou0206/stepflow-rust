use crate::app_state::AppState;
use serde_json::Value;
use stepflow_dto::dto::execution::ExecStart;
use stepflow_dto::dto::signal::ExecutionSignal;
use stepflow_engine::engine::WorkflowEngine;
use tracing::{debug, error, info, warn};
pub async fn handle_task_finished(
    app: &AppState,
    run_id: &str,
    state_name: &str,
    output: &serde_json::Value,
) {
    info!(%run_id, %state_name, "📩 Received TaskFinished");

    let mut engines = app.engines.lock().await;
    let engine = match engines.get_mut(run_id) {
        Some(e) => e,
        None => {
            debug!(%run_id, "⚠️ No active engine found");
            return;
        }
    };

    let signal = ExecutionSignal::TaskCompleted {
        run_id: run_id.to_string(),
        state_name: state_name.to_string(),
        output: output.clone(),
    };

    if let Some(tx) = engine.get_signal_sender() {
        if let Err(e) = tx.send(signal) {
            error!(%run_id, ?e, "❌ Failed to send signal");
            return;
        }
    } else {
        error!(%run_id, "❌ Missing signal sender");
        return;
    }

    info!(%run_id, %state_name, "✅ Signal sent, advancing workflow...");

    match engine.handle_next_signal().await {
        Ok(Some(result)) => {
            if result.should_continue && !result.is_blocking {
                if let Err(e) = engine.advance_until_blocked().await {
                    error!(%run_id, ?e, "❌ advance_until_blocked failed");
                    return;
                }
                info!(%run_id, "✅ advance_until_blocked complete");
            } else {
                if !result.should_continue && !result.is_blocking {
                    // ✅ 检查是否是 end 状态
                    if engine.dsl.is_end_state(&engine.current_state) {
                        if let Err(e) = engine.finalize().await {
                            error!(%run_id, ?e, "❌ finalize failed");
                            return;
                        }
                        info!(%run_id, "🏁 workflow finished via signal");
                    } else {
                        warn!(%run_id, "⚠️ Signal indicates termination, but current state is not end: {}", engine.current_state);
                    }
                } else {
                    debug!(%run_id, "🛑 Signal handled, but engine is now blocked or finished.");
                }
            }
        }
        Ok(None) => {
            debug!(%run_id, "⚠️ No signal to handle for run_id={}", run_id);
        }
        Err(e) => {
            error!(%run_id, ?e, "❌ handle_next_signal failed");
        }
    }

    // info!(%run_id, %state_name, "✅ Signal sent, advancing workflow...");
    // if let Err(e) = engine.handle_next_signal().await {
    //     error!(%run_id, ?e, "❌ handle_next_signal failed");
    //     return;
    // }

    // if let Err(e) = engine.advance_until_blocked().await {
    //     error!(%run_id, ?e, "❌ advance_until_blocked failed");
    //     return;
    // }
    // info!(%run_id, "✅ advance_until_blocked complete");

    if engine.finished {
        engines.remove(run_id);
        info!(%run_id, "🏁 workflow finished, engine removed");
    }
}

pub async fn handle_subflow_finished(
    app: &AppState,
    parent_run_id: &str,
    child_run_id: &str,
    state_name: &str,
    result: &serde_json::Value,
) {
    info!(%parent_run_id, %child_run_id, %state_name, "📩 Received SubflowFinished");

    let mut engines = app.engines.lock().await;
    let engine = match engines.get_mut(parent_run_id) {
        Some(e) => e,
        None => {
            debug!(%parent_run_id, "⚠️ No active parent engine found");
            return;
        }
    };

    let signal = ExecutionSignal::SubflowFinished {
        parent_run_id: parent_run_id.to_string(),
        child_run_id: child_run_id.to_string(),
        state_name: state_name.to_string(),
        result: result.clone(),
    };

    if let Some(tx) = engine.get_signal_sender() {
        if let Err(e) = tx.send(signal) {
            error!(%parent_run_id, ?e, "❌ Failed to send subflow signal");
            return;
        }
    } else {
        error!(%parent_run_id, "❌ Missing signal sender");
        return;
    }

    info!(%parent_run_id, %state_name, "✅ Subflow signal sent");

    if let Err(e) = engine.handle_next_signal().await {
        error!(%parent_run_id, ?e, "❌ handle_next_signal failed");
        return;
    }
    // if let Err(e) = engine.advance_until_blocked().await {
    //     error!(%parent_run_id, ?e, "❌ advance_until_blocked failed");
    //     return;
    // }

    if engine.finished {
        engines.remove(parent_run_id);
        info!(%parent_run_id, "🏁 parent workflow finished, engine removed");
    }
}

pub async fn handle_subflow_ready(
    app: &AppState,
    run_id: &str,
    parent_run_id: &str,
    state_name: &str,
    dsl: &Value,
    init_ctx: &Value,
) {
    info!(%run_id, %parent_run_id, %state_name, "📦 SubflowReady received");

    // 1. 是否已存在引擎
    {
        let engines = app.engines.lock().await;
        if engines.contains_key(run_id) {
            debug!(%run_id, "⚠️ Engine already exists, skipping");
            return;
        }
    }

    // 2. 是否已有执行记录
    let exists = match app.persist.get_execution(run_id).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            error!(%run_id, ?e, "❌ Failed to check execution existence");
            return;
        }
    };

    if exists {
        // 3. 已存在记录 → 尝试 restore
        match WorkflowEngine::restore(
            run_id.to_string(),
            app.event_dispatcher.clone(),
            app.persist.clone(),
            app.state_handler_registry.clone(),
        )
        .await
        {
            Ok(engine) => {
                if engine.finished {
                    info!(%run_id, "⏭️ Subflow already finished, skipping advance");
                    return;
                }

                info!(%run_id, "✅ Subflow engine restored");
                {
                    let mut engines = app.engines.lock().await;
                    engines.insert(run_id.to_string(), engine);
                }

                // ✅ restore 成功后再推进（避免 use-after-move）
                let mut engines = app.engines.lock().await;
                if let Some(engine) = engines.get_mut(run_id) {
                    match engine.advance_until_blocked().await {
                        Ok(_) => info!(%run_id, "✅ Subflow advanced after restore"),
                        Err(e) => error!(%run_id, "❌ Subflow advance failed: {}", e),
                    }
                }
            }

            Err(e) => {
                error!(%run_id, "❌ Failed to restore subflow engine: {}", e);
                return;
            }
        }
    } else {
        // 4. fallback（理论上不会走到这里）
        warn!(%run_id, "⚠️ Subflow not found in DB, fallback to start()");
        let req = ExecStart {
            run_id: Some(run_id.to_string()),
            template_id: None,
            init_ctx: Some(init_ctx.clone()),
            parent_run_id: Some(parent_run_id.to_string()),
            parent_state_name: Some(state_name.to_string()),
            dsl: Some(dsl.clone()),
        };

        if let Err(e) = app.services.execution.start(req).await {
            error!(%run_id, "❌ Failed to fallback-start subflow: {e:#}");
        } else {
            info!(%run_id, "✅ Subflow started via fallback");
        }
    }
}

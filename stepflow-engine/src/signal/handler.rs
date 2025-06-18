// ✅ 扩展 apply_signal：准备支持 Map / Parallel 的 SubflowFinished 聚合
use stepflow_dsl::State;
use stepflow_dto::dto::signal::ExecutionSignal;
use crate::engine::WorkflowEngine;
use crate::handler::execution_scope::StateExecutionResult;
use stepflow_storage::entities::workflow_execution::UpdateStoredWorkflowExecution;
use chrono::Utc;

/// 应用信号并推进引擎，返回 StepExecutionResult
pub async fn apply_signal(
    engine: &mut WorkflowEngine,
    signal: ExecutionSignal,
) -> Result<StateExecutionResult, String> {
    match signal {
        ExecutionSignal::TaskCompleted { run_id, state_name, output } => {
            if run_id != engine.run_id {
                return Err("Signal mismatch: wrong run_id".into());
            }
            if state_name != engine.current_state && Some(state_name.clone()) != engine.last_task_state {
                return Err(format!(
                    "Signal state_name '{}' does not match current state '{}' or last task state {:?}",
                    state_name, engine.current_state, engine.last_task_state
                ));
            }

            let state = if matches!(engine.state_def(), State::Task(_)) {
                engine.state_def()
            } else if let Some(last_task) = &engine.last_task_state {
                &engine.dsl.states[last_task]
            } else {
                return Err("No Task state found for signal".into());
            };

            let State::Task(task_state) = state else {
                return Err("TaskCompleted signal applied to non-Task state".into());
            };

            let pipeline = crate::mapping::MappingPipeline {
                input_mapping: task_state.base.input_mapping.as_ref(),
                output_mapping: task_state.base.output_mapping.as_ref(),
            };

            engine.context = pipeline
                .apply_output(&output, &engine.context)
                .map_err(|e| format!("output mapping failed: {e}"))?;

            engine.dispatch_event(stepflow_dto::dto::engine_event::EngineEvent::NodeSuccess {
                run_id,
                state_name,
                output: output.clone(),
            }).await;

            Ok(StateExecutionResult {
                output: engine.context.clone(),
                next_state: Some(engine.current_state.clone()),
                should_continue: true,
                metadata: Some(output),
            })
        }

        ExecutionSignal::TaskFailed { run_id, state_name, error } => {
            if run_id != engine.run_id {
                return Err("Signal mismatch: wrong run_id".into());
            }
            if state_name != engine.current_state && Some(state_name.clone()) != engine.last_task_state {
                return Err(format!(
                    "Signal state_name '{}' does not match current state '{}' or last task state {:?}",
                    state_name, engine.current_state, engine.last_task_state
                ));
            }

            engine.dispatch_event(stepflow_dto::dto::engine_event::EngineEvent::NodeFailed {
                run_id,
                state_name,
                error: error.clone(),
            }).await;

            Err(format!("Task failed: {}", error))
        }

        ExecutionSignal::TaskCancelled { run_id, state_name, reason } => {
            if run_id != engine.run_id {
                return Err("Signal mismatch: wrong run_id".into());
            }
            if state_name != engine.current_state && Some(state_name.clone()) != engine.last_task_state {
                return Err(format!(
                    "Signal state_name '{}' does not match current state '{}' or last task state {:?}",
                    state_name, engine.current_state, engine.last_task_state
                ));
            }

            let state = if matches!(engine.state_def(), State::Task(_)) {
                engine.state_def()
            } else if let Some(last_task) = &engine.last_task_state {
                &engine.dsl.states[last_task]
            } else {
                return Err("No Task state found for signal".into());
            };

            let State::Task(_) = state else {
                return Err("TaskCancelled signal applied to non-Task state".into());
            };

            engine.persistence
                .update_execution(
                    &run_id,
                    &UpdateStoredWorkflowExecution {
                        status: Some("CANCELLED".into()),
                        close_time: Some(Some(Utc::now().naive_utc())),
                        ..Default::default()
                    },
                )
                .await
                .map_err(|e| e.to_string())?;

            engine.dispatch_event(stepflow_dto::dto::engine_event::EngineEvent::NodeCancelled {
                run_id: run_id.clone(),
                state_name: state_name.clone(),
                reason: reason.clone().unwrap_or_else(|| "Task cancelled".to_string()),
            }).await;

            engine.finished = true;

            Err(format!("Task cancelled: {}", reason.unwrap_or_else(|| "Task cancelled".to_string())))
        }

        ExecutionSignal::TimerFired { .. } => {
            Err("TimerFired signal not yet supported".into())
        }

        ExecutionSignal::Heartbeat { .. } => {
            Err("Heartbeat signal not yet supported".into())
        }

        ExecutionSignal::SubflowFinished { .. } => {
            Err("SubflowFinished signal not yet supported".into())
        }
    }
}
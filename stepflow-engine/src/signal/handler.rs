use stepflow_dsl::State;
use stepflow_dto::dto::signal::ExecutionSignal;
use crate::engine::WorkflowEngine;
use crate::handler::execution_scope::StateExecutionScope;
use crate::handler::execution_scope::StateExecutionResult;
use stepflow_storage::entities::workflow_execution::UpdateStoredWorkflowExecution;
use chrono::Utc;

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
                reason: reason.unwrap_or_else(|| "Task cancelled".into()),
            }).await;

            engine.finished = true;

            Err("Task cancelled".into())
        }

        ExecutionSignal::TimerFired { .. } => {
            Err("TimerFired signal not yet supported".into())
        }

        ExecutionSignal::Heartbeat { .. } => {
            Err("Heartbeat signal not yet supported".into())
        }

        ExecutionSignal::SubflowFinished {
            parent_run_id,
            child_run_id,
            state_name,
            result,
        } => {
            tracing::debug!(
                "[signal] SubflowFinished received: parent_run_id={}, child_run_id={}, state_name={}, result={:?}",
                parent_run_id, child_run_id, state_name, result
            );

            if parent_run_id != engine.run_id {
                return Err("SubflowFinished: parent_run_id mismatch".into());
            }
            if state_name != engine.current_state {
                return Err(format!(
                    "SubflowFinished: state_name mismatch '{}', expected '{}'",
                    state_name, engine.current_state
                ));
            }

            let state_type = engine.state_def().variant_name();
            tracing::debug!(
                "[signal] Dispatching on_subflow_finished for state_type={}",
                state_type
            );

            let handler = engine
                .state_handler_registry
                .get(state_type)
                .ok_or_else(|| format!("No handler registered for state type: {state_type}"))?;

            let scope = StateExecutionScope::new(
                &engine.run_id,
                &engine.current_state,
                state_type,
                engine.mode,
                Some(&engine.event_dispatcher),
                &engine.persistence,
                engine.state_def(),
                &engine.context,
            );

            let result = handler
                .on_subflow_finished(&scope, &engine.context, &child_run_id, &result)
                .await?;

            tracing::debug!(
                "[signal] on_subflow_finished result: should_continue={}, next_state={:?}",
                result.should_continue, result.next_state
            );

            engine.context = result.output.clone();

            if result.should_continue {
                let next = result
                    .next_state
                    .clone()
                    .ok_or("Missing next_state in subflow result")?;

                engine.current_state = next.clone();

                tracing::debug!(
                    "[signal] Advancing to next_state={}",
                    next
                );

                engine.persistence
                    .update_execution(&engine.run_id, &UpdateStoredWorkflowExecution {
                        current_state_name: Some(Some(next)),
                        context_snapshot: Some(Some(engine.context.clone())),
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| e.to_string())?;
            }

            Ok(result)
        }
    
    }
}
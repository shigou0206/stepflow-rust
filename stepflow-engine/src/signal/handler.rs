use stepflow_dsl::State;
use stepflow_dto::dto::signal::ExecutionSignal;
use crate::engine::WorkflowEngine;
use crate::handler::execution_scope::StateExecutionResult;

/// 应用信号并推进引擎，返回 StepExecutionResult
pub async fn apply_signal(
    engine: &mut WorkflowEngine,
    signal: ExecutionSignal,
) -> Result<StateExecutionResult, String> {
    match signal {
        ExecutionSignal::TaskCompleted {
            run_id,
            state_name,
            output,
        } => {
            if run_id != engine.run_id || state_name != engine.current_state {
                return Err("Signal mismatch: wrong run_id or state_name".into());
            }

            let state = engine.state_def();
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

            // ✅ 先补发 NodeEnter（为了持久化/日志一致）
            engine.dispatch_event(stepflow_dto::dto::engine_event::EngineEvent::NodeEnter {
                run_id: run_id.clone(),
                state_name: state_name.clone(),
                input: output.clone(),
            }).await;

            // ✅ 再发送 NodeSuccess
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

        ExecutionSignal::TaskFailed {
            run_id,
            state_name,
            error,
        } => {
            if run_id != engine.run_id || state_name != engine.current_state {
                return Err("Signal mismatch: wrong run_id or state_name".into());
            }

            engine.dispatch_event(stepflow_dto::dto::engine_event::EngineEvent::NodeFailed {
                run_id,
                state_name,
                error: error.clone(),
            }).await;

            Err(format!("Task failed: {}", error))
        }

        ExecutionSignal::TaskCancelled { .. } => {
            Err("TaskCancelled signal not yet supported".into())
        }

        ExecutionSignal::TimerFired { .. } => {
            Err("TimerFired signal not yet supported".into())
        }

        ExecutionSignal::Heartbeat { .. } => {
            Err("Heartbeat signal not yet supported".into())
        }
    }
}
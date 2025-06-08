use crate::{command::Command, handler, mapping::MappingPipeline};
use serde_json::Value;
use stepflow_dsl::{state::base::BaseState, State};
use stepflow_hook::{EngineEvent, EngineEventDispatcher};
use stepflow_match::service::MatchService;
use stepflow_storage::persistence_manager::PersistenceManager;

use super::types::{StepOutcome, WorkflowMode};
use std::{error, sync::Arc};
use thiserror::Error;
use log::error;
use tracing::info;

#[derive(Error, Debug)]
pub enum DispatchError {
    #[error("Task execution failed: {0}")]
    TaskError(String),
    #[error("State transition failed: {0}")]
    StateError(String),
    #[error("Input/Output mapping failed: {0}")]
    MappingError(String),
}

impl From<String> for DispatchError {
    fn from(s: String) -> Self {
        DispatchError::StateError(s)
    }
}

type DispatchResult<T> = Result<T, DispatchError>;

struct EventContext<'a> {
    run_id: &'a str,
    state_name: &'a str,
    dispatcher: &'a Arc<EngineEventDispatcher>,
}

impl<'a> EventContext<'a> {
    fn new(
        run_id: &'a str,
        state_name: &'a str,
        dispatcher: &'a Arc<EngineEventDispatcher>,
    ) -> Self {
        Self {
            run_id,
            state_name,
            dispatcher,
        }
    }

    async fn enter(&self, input: &Value) {
        self.dispatcher
            .dispatch(EngineEvent::NodeEnter {
                run_id: self.run_id.to_string(),
                state_name: self.state_name.to_string(),
                input: input.clone(),
            })
            .await;
    }

    async fn success(&self, output: &Value) {
        self.dispatcher
            .dispatch(EngineEvent::NodeSuccess {
                run_id: self.run_id.to_string(),
                state_name: self.state_name.to_string(),
                output: output.clone(),
            })
            .await;
    }

    async fn fail(&self, error: &str) {
        self.dispatcher
            .dispatch(EngineEvent::NodeFailed {
                run_id: self.run_id.to_string(),
                state_name: self.state_name.to_string(),
                error: error.to_string(),
            })
            .await;
    }
}

trait StateTransition {
    async fn execute(
        &self,
        state_name: &str,
        input: &Value,
        run_id: &str,
        mode: WorkflowMode,
        match_service: Arc<dyn MatchService>,
        persistence: &Arc<dyn PersistenceManager>,
        event_dispatcher: &Arc<EngineEventDispatcher>,
        persistence_for_handlers: &Arc<dyn PersistenceManager>,
    ) -> DispatchResult<(Value, Option<String>)>;
}

impl StateTransition for State {
    async fn execute(
        &self,
        state_name: &str,
        input: &Value,
        run_id: &str,
        mode: WorkflowMode,
        match_service: Arc<dyn MatchService>,
        persistence: &Arc<dyn PersistenceManager>,
        event_dispatcher: &Arc<EngineEventDispatcher>,
        persistence_for_handlers: &Arc<dyn PersistenceManager>,
    ) -> DispatchResult<(Value, Option<String>)> {
        let evt_ctx = EventContext::new(run_id, state_name, event_dispatcher);
        evt_ctx.enter(input).await;

        info!("🔥 executing state: {:?}", self);

        let result = match self {
            State::Task(t) => {
                match handler::handle_task(
                    mode,
                    run_id,
                    state_name,
                    t,
                    &input,
                    match_service,
                    persistence.clone(),
                )
                .await
                {
                    Ok(result) => {
                        evt_ctx.success(&result).await;
                        Ok((result, t.base.next.clone()))
                    }
                    Err(e) => {
                        evt_ctx.fail(&e).await;
                        Err(DispatchError::TaskError(e))
                    }
                }
            }

            State::Wait(w) => {
                match handler::handle_wait(
                    state_name,
                    w,
                    input,
                    mode,
                    run_id,
                    persistence_for_handlers,
                    event_dispatcher,
                )
                .await
                {
                    Ok(result) => {
                        evt_ctx.success(&result).await;
                        Ok((result, w.base.next.clone()))
                    }
                    Err(e) => {
                        evt_ctx.fail(&e).await;
                        Err(DispatchError::StateError(e))
                    }
                }
            }

            State::Pass(p) => {
                match handler::handle_pass(
                    state_name,
                    p,
                    input,
                    run_id,
                    event_dispatcher,
                    persistence,
                )
                .await
                {
                    Ok(result) => {
                        evt_ctx.success(&result).await;
                        Ok((result, p.base.next.clone()))
                    }
                    Err(e) => {
                        evt_ctx.fail(&e).await;
                        Err(DispatchError::StateError(e))
                    }
                }
            }

            State::Choice(c) => {
                match handler::handle_choice(
                    state_name,
                    c,
                    input,
                    run_id,
                    event_dispatcher,
                    persistence,
                )
                .await
                {
                    Ok(result) => {
                        evt_ctx.success(&result).await;
                        Ok((result, None))
                    }
                    Err(e) => {
                        evt_ctx.fail(&e).await;
                        Err(DispatchError::StateError(e.to_string()))
                    }
                }
            }

            State::Succeed(s) => {
                match handler::handle_succeed(
                    state_name,
                    s,
                    input,
                    run_id,
                    event_dispatcher,
                    persistence,
                )
                .await
                {
                    Ok(result) => {
                        evt_ctx.success(&result).await;
                        Ok((result, None))
                    }
                    Err(e) => {
                        evt_ctx.fail(&e).await;
                        Err(DispatchError::StateError(e))
                    }
                }
            }

            State::Fail(f) => {
                match handler::handle_fail(
                    state_name,
                    f,
                    input,
                    run_id,
                    event_dispatcher,
                    persistence,
                )
                .await
                {
                    Ok(result) => {
                        evt_ctx.success(&result).await;
                        Ok((result, None))
                    }
                    Err(e) => {
                        evt_ctx.fail(&e).await;
                        Err(DispatchError::StateError(e))
                    }
                }
            }

            _ => {
                let err = "Parallel / Map not yet supported".to_string();
                evt_ctx.fail(&err).await;
                Err(DispatchError::StateError(err))
            }
        }?;

        Ok(result)
    }
}

pub(crate) async fn dispatch_command(
    cmd: &Command,
    state_enum: &State,
    context: &Value,
    run_id: &str,
    mode: WorkflowMode,
    match_service: Arc<dyn MatchService>,
    persistence: &Arc<dyn PersistenceManager>,
    event_dispatcher: Arc<EngineEventDispatcher>,
    persistence_for_handlers: Arc<dyn PersistenceManager>,
) -> Result<(StepOutcome, Option<String>), String> {
    let state_name = cmd.state_name().to_string();

    let base: &BaseState = match state_enum {
        State::Task(s) => &s.base,
        State::Wait(s) => &s.base,
        State::Pass(s) => &s.base,
        State::Choice(s) => &s.base,
        State::Fail(s) => &s.base,
        State::Succeed(s) => &s.base,
        State::Parallel(_) | State::Map(_) => {
            return Err("Parallel / Map not yet supported".to_string());
        }
    };

    let pipeline = MappingPipeline {
        input_mapping: base.input_mapping.as_ref(),
        output_mapping: base.output_mapping.as_ref(),
    };

    let _ = dbg!(format!("🟡 dispatch_command entered with state: {}", state_name));
    // ✅ input + parameters 已合并
    let exec_in = match pipeline.apply_input(context) {
        Ok(val) => val,
        Err(e) => {
            error!("❌ apply_input failed for state {}: {:?}", state_name, e);
            return Err(format!("apply_input failed: {:?}", e));
        }
    };

    let _ = dbg!(format!("🟢 exec_in ready: {:?}", &exec_in));
    
    let (raw_out, mut logical_next) = state_enum
        .execute(
            &state_name,
            &exec_in,
            run_id,
            mode,
            match_service,
            persistence,
            &event_dispatcher,
            &persistence_for_handlers,
        )
        .await
        .map_err(|e| e.to_string())?;

    if let (Command::Choice { next_state, .. }, State::Choice(_)) = (cmd, state_enum) {
        logical_next = Some(next_state.clone());
    }

    let new_ctx = pipeline
        .apply_output(&raw_out, context)
        .map_err(|e| DispatchError::MappingError(e.to_string()))
        .map_err(|e| e.to_string())?;

    Ok((
        StepOutcome {
            should_continue: logical_next.is_some(),
            updated_context: new_ctx,
        },
        logical_next,
    ))
}

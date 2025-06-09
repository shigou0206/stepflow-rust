use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use thiserror::Error;

use stepflow_dsl::state::wait::WaitState;
use stepflow_storage::db::DynPM;
use stepflow_storage::entities::timer::StoredTimer;
use stepflow_hook::{EngineEvent, EngineEventDispatcher};

use crate::{
    engine::WorkflowMode,
    mapping::MappingPipeline,
};
use super::{StateHandler, StateExecutionContext, StateExecutionResult};

const MAX_INLINE_WAIT_SECONDS: u64 = 300;

#[derive(Error, Debug)]
pub enum WaitError {
    #[error("Timestamp-based wait not yet supported")]
    TimestampNotSupported,
    #[error("Wait time too long for inline mode: {0} seconds")]
    WaitTooLong(u64),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub struct WaitHandler<'a> {
    state: &'a WaitState,
}

impl<'a> WaitHandler<'a> {
    pub fn new(state: &'a WaitState) -> Self {
        Self { state }
    }

    async fn handle_inline(&self, secs: u64) -> Result<(), String> {
        if secs > MAX_INLINE_WAIT_SECONDS {
            return Err(WaitError::WaitTooLong(secs).to_string());
        }
        info!("⏳ Inline wait for {} seconds", secs);
        tokio::time::sleep(Duration::from_secs(secs)).await;
        debug!("✅ Inline wait complete");
        Ok(())
    }

    async fn handle_deferred(
        &self,
        ctx: &StateExecutionContext<'_>,
        secs: u64,
    ) -> Result<(), String> {
        let now = Utc::now().naive_utc();
        let fire_at = now + chrono::Duration::seconds(secs as i64);

        let timer = StoredTimer {
            timer_id: Uuid::new_v4().to_string(),
            run_id: ctx.run_id.to_string(),
            state_name: Some(ctx.state_name.to_string()),
            fire_at,
            shard_id: 0,
            version: 1,
            status: "pending".to_string(),
            payload: None,
            created_at: now,
            updated_at: now,
        };

        ctx.dispatcher.dispatch(EngineEvent::NodeDispatched {
            run_id: ctx.run_id.to_string(),
            state_name: ctx.state_name.to_string(),
            context: json!({
                "timer_id": timer.timer_id,
                "fire_at": fire_at,
                "mode": "deferred"
            }),
        }).await;

        ctx.persistence
            .create_timer(&timer)
            .await
            .map_err(|e| WaitError::DatabaseError(e.to_string()).to_string())?;

        info!("🕒 Deferred timer created to fire at {}", fire_at);
        Ok(())
    }

    async fn process_wait(
        &self,
        ctx: &StateExecutionContext<'_>,
        _exec_input: &Value,
    ) -> Result<(), String> {
        if let Some(secs) = self.state.seconds {
            if secs == 0 {
                debug!("⏩ Wait = 0s, skipping wait");
                return Ok(());
            }

            match ctx.mode {
                WorkflowMode::Inline => self.handle_inline(secs).await?,
                WorkflowMode::Deferred => self.handle_deferred(ctx, secs).await?,
            }
        } else if self.state.timestamp.is_some() {
            warn!("Timestamp wait specified, but not supported yet");
            return Err(WaitError::TimestampNotSupported.to_string());
        }

        Ok(())
    }
}

#[async_trait]
impl<'a> StateHandler for WaitHandler<'a> {
    async fn handle(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let pipeline = MappingPipeline {
            input_mapping: self.state.base.input_mapping.as_ref(),
            output_mapping: self.state.base.output_mapping.as_ref(),
        };

        let exec_input = pipeline.apply_input(input)?;

        debug!("WaitHandler input mapped: {}", exec_input);

        self.process_wait(ctx, &exec_input).await?;

        let final_output = pipeline.apply_output(&exec_input, input)?;

        debug!("WaitHandler final output: {}", final_output);

        Ok(StateExecutionResult {
            output: final_output,
            next_state: self.state.base.next.clone(),
            should_continue: true,
        })
    }

    fn state_type(&self) -> &'static str {
        "wait"
    }
}

pub async fn handle_wait(
    state_name: &str,
    state: &WaitState,
    input: &Value,
    mode: WorkflowMode,
    run_id: &str,
    persistence: &DynPM,
    event_dispatcher: &Arc<EngineEventDispatcher>,
) -> Result<Value, String> {
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        "wait",
        mode,
        event_dispatcher,
        persistence,
    );

    let handler = WaitHandler::new(state);
    let result = handler.execute(&ctx, input).await?;
    Ok(result.output)
}
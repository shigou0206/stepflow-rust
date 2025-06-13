use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use thiserror::Error;

use stepflow_dsl::state::wait::WaitState;
use stepflow_storage::db::DynPM;
use stepflow_storage::entities::timer::StoredTimer;
use stepflow_dto::dto::timer::TimerDto;

use crate::{
    engine::WorkflowMode,
    mapping::MappingPipeline,
};
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

use chrono::DateTime;

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
        scope: &StateExecutionScope<'_>,
        secs: u64,
    ) -> Result<Value, String> {
        let now = Utc::now().naive_utc();
        let fire_at = now + chrono::Duration::seconds(secs as i64);

        let timer = StoredTimer {
            timer_id: Uuid::new_v4().to_string(),
            run_id: scope.run_id.to_string(),
            state_name: Some(scope.state_name.to_string()),
            fire_at,
            shard_id: 0,
            version: 1,
            status: "pending".to_string(),
            payload: None,
            created_at: now,
            updated_at: now,
        };

        scope.persistence
            .create_timer(&timer)
            .await
            .map_err(|e| WaitError::DatabaseError(e.to_string()).to_string())?;

        info!("🕒 Deferred timer created to fire at {}", fire_at);

        let metadata = serde_json::to_value(&TimerDto {
            timer_id: timer.timer_id,
            run_id: timer.run_id,
            shard_id: timer.shard_id,
            fire_at: DateTime::<Utc>::from_utc(timer.fire_at, Utc),
            status: timer.status,
            version: timer.version,
            state_name: timer.state_name,
            payload: timer.payload,
            created_at: DateTime::<Utc>::from_utc(timer.created_at, Utc),
            updated_at: DateTime::<Utc>::from_utc(timer.updated_at, Utc),
        }).map_err(|e| format!("Failed to serialize timer metadata: {e}"))?;

        Ok(metadata)
    }

    async fn process_wait(
        &self,
        scope: &StateExecutionScope<'_>,
        _exec_input: &Value,
    ) -> Result<Option<Value>, String> {
        if let Some(secs) = self.state.seconds {
            if secs == 0 {
                debug!("⏩ Wait = 0s, skipping wait");
                return Ok(None);
            }

            match scope.mode {
                WorkflowMode::Inline => {
                    self.handle_inline(secs).await?;
                    Ok(None)
                },
                WorkflowMode::Deferred => {
                    let metadata = self.handle_deferred(scope, secs).await?;
                    Ok(Some(metadata))
                },
            }
        } else if self.state.timestamp.is_some() {
            warn!("Timestamp wait specified, but not supported yet");
            Err(WaitError::TimestampNotSupported.to_string())
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl<'a> StateHandler for WaitHandler<'a> {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let pipeline = MappingPipeline {
            input_mapping: self.state.base.input_mapping.as_ref(),
            output_mapping: self.state.base.output_mapping.as_ref(),
        };

        let exec_input = pipeline.apply_input(input)?;

        debug!("WaitHandler input mapped: {}", exec_input);

        let metadata = self.process_wait(scope, &exec_input).await?;

        let final_output = pipeline.apply_output(&exec_input, input)?;

        debug!("WaitHandler final output: {}", final_output);

        Ok(StateExecutionResult {
            output: final_output,
            next_state: self.state.base.next.clone(),
            should_continue: true,
            metadata,
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
) -> Result<Value, String> {
    let scope = StateExecutionScope::new(
        run_id,
        state_name,
        "wait",
        mode,
        None,
        persistence,
    );

    let handler = WaitHandler::new(state);
    let result = handler.handle(&scope, input).await?;
    Ok(result.output)
}
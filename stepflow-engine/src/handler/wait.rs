use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

use stepflow_dsl::state::{wait::WaitState, State};
use stepflow_dto::dto::timer::TimerDto;
use stepflow_storage::entities::timer::StoredTimer;

use crate::mapping::MappingPipeline;
use super::{StateExecutionResult, StateExecutionScope, StateHandler};

/// ---------------------------------------------------------------------
/// WaitHandler：用于实现延迟等待的状态（DEFERRED）
/// ---------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum WaitError {
    #[error("Timestamp-based wait not yet supported")]
    TimestampNotSupported,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub struct WaitHandler;

impl WaitHandler {
    pub fn new() -> Self {
        Self
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

        scope
            .persistence
            .create_timer(&timer)
            .await
            .map_err(|e| WaitError::DatabaseError(e.to_string()).to_string())?;

        info!(run_id = scope.run_id, fire_at = %fire_at, "🕒 Timer scheduled");

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
        })
        .map_err(|e| format!("Failed to serialize timer metadata: {e}"))?;

        Ok(metadata)
    }

    async fn process_wait(
        &self,
        scope: &StateExecutionScope<'_>,
        state: &WaitState,
    ) -> Result<Option<Value>, String> {
        if let Some(secs) = state.seconds {
            if secs == 0 {
                debug!(run_id = scope.run_id, "⏩ seconds = 0, skip wait");
                return Ok(None);
            }

            self.handle_deferred(scope, secs).await.map(Some)
        } else if state.timestamp.is_some() {
            warn!(run_id = scope.run_id, "⏱ timestamp wait not implemented");
            Err(WaitError::TimestampNotSupported.to_string())
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl StateHandler for WaitHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Wait(ref s) => s,
            _ => return Err("Invalid state type for WaitHandler".into()),
        };

        // ✅ 参数映射（仅 input_mapping）
        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: None,
        };

        let exec_input = pipeline.apply_input(&scope.context)?;
        debug!(run_id = scope.run_id, state = scope.state_name, input = ?exec_input, "WaitHandler input");

        let metadata = self.process_wait(scope, state).await?;

        Ok(StateExecutionResult {
            output: exec_input,
            next_state: state.base.next.clone(),
            should_continue: false,
            metadata,
            is_blocking: true,
        })
    }

    fn state_type(&self) -> &'static str {
        "wait"
    }

    async fn on_subflow_finished(
        &self,
        _scope: &StateExecutionScope<'_>,
        _parent_context: &Value,
        _child_run_id: &str,
        _result: &Value,
    ) -> Result<StateExecutionResult, String> {
        Err("WaitHandler does not support subflow".into())
    }
}
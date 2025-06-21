use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

use stepflow_dsl::state::{wait::WaitState, State};
use stepflow_dto::dto::timer::TimerDto;
use stepflow_storage::entities::timer::StoredTimer;

use super::{StateExecutionResult, StateExecutionScope, StateHandler};

#[derive(Error, Debug)]
pub enum WaitError {
    #[error("Timestamp-based wait not yet supported")]
    TimestampNotSupported,
    #[error("Wait time too long for inline mode: {0} seconds")]
    WaitTooLong(u64),
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
        })
        .map_err(|e| format!("Failed to serialize timer metadata: {e}"))?;

        Ok(metadata)
    }

    async fn process_wait(
        &self,
        scope: &StateExecutionScope<'_>,
        state: &WaitState,
        _exec_input: &Value,
    ) -> Result<Option<Value>, String> {
        if let Some(secs) = state.seconds {
            if secs == 0 {
                debug!("⏩ Wait = 0s, skipping wait");
                return Ok(None);
            }

            let metadata = self.handle_deferred(scope, secs).await?;
            Ok(Some(metadata))
        } else if state.timestamp.is_some() {
            warn!("Timestamp wait specified, but not supported yet");
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
        input: &Value, // 这是已经经过 input_mapping 后的执行输入
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Wait(ref w) => w,
            _ => return Err("Invalid state type for WaitHandler".into()),
        };

        debug!(run_id = scope.run_id, state = scope.state_name, "WaitHandler input: {}", input);

        let metadata = self.process_wait(scope, state, input).await?;

        // ✅ output_mapping 将由 dispatch_command 统一处理，handler 只需返回 input
        Ok(StateExecutionResult {
            output: input.clone(),
            next_state: state.base.next.clone(),
            should_continue: true,
            metadata,
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
        Err("on_subflow_finished not supported by this state".into())
    }
}
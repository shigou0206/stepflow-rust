use std::time::Duration;
use tokio::time::sleep;
use serde_json::Value;
use tracing::{debug, info, warn};
use thiserror::Error;
use chrono::Utc;
use uuid::Uuid;
use stepflow_dsl::state::wait::WaitState;
use stepflow_sqlite::models::timer::Timer;
use crate::engine::WorkflowMode;
use async_trait::async_trait;
use super::{StateHandler, StateExecutionContext, StateExecutionResult};
use serde_json::json;
use stepflow_storage::persistence_manager::PersistenceManager;
const MAX_INLINE_WAIT_SECONDS: u64 = 300; // 5分钟

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
            let err = WaitError::WaitTooLong(secs);
            warn!("Wait time too long for inline mode: {}", err);
            return Err(err.to_string());
        }
        
        info!("Inline waiting for {} seconds", secs);
        sleep(Duration::from_secs(secs)).await;
        debug!("Inline wait completed");
        
        Ok(())
    }

    async fn handle_deferred(
        &self,
        ctx: &StateExecutionContext<'_>,
        secs: u64,
    ) -> Result<(), String> {
        let now = Utc::now().naive_utc();
        let trigger_at = now + chrono::Duration::seconds(secs as i64);
        
        let timer = Timer {
            timer_id: Uuid::new_v4().to_string(),
            run_id: ctx.run_id.to_string(),
            state_name: Some(ctx.state_name.to_string()),
            fire_at: trigger_at,
            shard_id: 0,  // 默认分片
            version: 1,   // 初始版本
            status: "pending".to_string(),
            payload: None,
            created_at: now,
            updated_at: now,
        };

        // 分发定时器创建事件
        ctx.dispatcher.dispatch(stepflow_hook::EngineEvent::NodeDispatched {
            run_id: ctx.run_id.to_string(),
            state_name: ctx.state_name.to_string(),
            context: json!({
                "timer_id": timer.timer_id,
                "fire_at": timer.fire_at,
                "mode": "deferred"
            }),
        }).await;

        ctx.persistence.create_timer(&timer)
            .await
            .map_err(|e| WaitError::DatabaseError(e.to_string()).to_string())?;

        info!(
            "Created deferred timer, will trigger at {}",
            trigger_at
        );
        
        Ok(())
    }

    async fn process_wait(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<Value, String> {
        if let Some(secs) = self.state.seconds {
            if secs == 0 {
                debug!("Zero second wait, continuing immediately");
                return Ok(input.clone());
            }

            match ctx.mode {
                WorkflowMode::Inline => {
                    self.handle_inline(secs).await?;
                }
                WorkflowMode::Deferred => {
                    self.handle_deferred(ctx, secs).await?;
                }
            }
        } else if self.state.timestamp.is_some() {
            let err = WaitError::TimestampNotSupported;
            debug!("Timestamp wait not supported: {}", err);
            return Err(err.to_string());
        }

        Ok(input.clone())
    }
}

#[async_trait]
impl<'a> StateHandler for WaitHandler<'a> {
    async fn handle(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let output = self.process_wait(ctx, input).await?;

        Ok(StateExecutionResult {
            output,
            next_state: self.state.base.next.clone(),
            should_continue: true,
        })
    }

    fn state_type(&self) -> &'static str {
        "wait"
    }
}

// 为了保持向后兼容，保留原有的函数签名
pub async fn handle_wait(
    state_name: &str,
    state: &WaitState,
    input: &Value,
    mode: WorkflowMode,
    run_id: &str,
    persistence: &std::sync::Arc<dyn PersistenceManager>,
    event_dispatcher: &std::sync::Arc<stepflow_hook::EngineEventDispatcher>,
) -> Result<Value, String> {
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        mode,
        event_dispatcher,
        persistence,
    );

    let handler = WaitHandler::new(state);
    let result = handler.execute(&ctx, input).await?;
    
    Ok(result.output)
}

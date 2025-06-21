use std::sync::Arc;
use async_trait::async_trait;

use crate::app_state::AppState;
use crate::service::WorkflowEngineService;
use crate::error::{AppResult, AppError};

use stepflow_engine::engine::WorkflowEngine;
use stepflow_dto::dto::signal::ExecutionSignal;
use stepflow_dto::dto::engine::{EngineStatusDto, ControlRequest, RetryRequest};
use stepflow_dto::dto::timer::TimerDto;

pub struct LocalEngineService {
    pub app: Arc<AppState>,
}

impl LocalEngineService {
    pub fn new(app: Arc<AppState>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl WorkflowEngineService for LocalEngineService {
    async fn send_subflow_finished(&self, run_id: &str, state_name: &str) -> AppResult<()> {
        let mut engines = self.app.engines.lock().await;
        let engine = if let Some(engine) = engines.get_mut(run_id) {
            engine
        } else {
            let restored = WorkflowEngine::restore(
                run_id.to_string(),
                self.app.event_dispatcher.clone(),
                self.app.persist.clone(),
                self.app.state_handler_registry.clone(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Restore engine failed: {e}")))?;
            engines.insert(run_id.to_string(), restored);
            engines.get_mut(run_id).unwrap()
        };

        if let Some(tx) = engine.get_signal_sender() {
            tx.send(ExecutionSignal::SubflowFinished {
                parent_run_id: run_id.to_string(),
                child_run_id: state_name.to_string(),
                state_name: state_name.to_string(),
                result: engine.context.clone(),
            })
            .map_err(|e| AppError::Internal(format!("Signal send failed: {e}")))?;
        }

        engine.handle_next_signal().await.map_err(AppError::Internal)?;
        Ok(())
    }

    async fn handle_timer_fired(&self, timer: &TimerDto) -> AppResult<()> {
        let mut engines = self.app.engines.lock().await;
        let run_id = &timer.run_id;
        let state_name = &timer.state_name;

        let engine = if let Some(engine) = engines.get_mut(run_id) {
            engine
        } else {
            let restored = WorkflowEngine::restore(
                run_id.clone(),
                self.app.event_dispatcher.clone(),
                self.app.persist.clone(),
                self.app.state_handler_registry.clone(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Restore engine failed: {e}")))?;
            engines.insert(run_id.clone(), restored);
            engines.get_mut(run_id).unwrap()
        };

        if let Some(tx) = engine.get_signal_sender() {
            tx.send(ExecutionSignal::TimerFired {
                run_id: run_id.clone(),
                state_name: state_name.clone().unwrap_or_default(),
            })
            .map_err(|e| AppError::Internal(format!("Timer signal send failed: {e}")))?;
        }

        engine.handle_next_signal().await.map_err(AppError::Internal)?;
        Ok(())
    }

    async fn cancel(&self, req: ControlRequest) -> AppResult<()> {
        self.cleanup(req).await
    }

    async fn terminate(&self, req: ControlRequest) -> AppResult<()> {
        self.cleanup(req).await
    }

    async fn pause(&self, _req: ControlRequest) -> AppResult<()> {
        Err(AppError::NotImplemented("pause not implemented".into()))
    }

    async fn resume(&self, _req: ControlRequest) -> AppResult<()> {
        Err(AppError::NotImplemented("resume not implemented".into()))
    }

    async fn retry_failed(&self, _req: RetryRequest) -> AppResult<()> {
        Err(AppError::NotImplemented("retry_failed not implemented".into()))
    }

    async fn list_running(&self) -> AppResult<Vec<String>> {
        let rows = self
            .app
            .persist
            .find_executions_by_status("RUNNING", 1000, 0)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(rows.into_iter().map(|r| r.run_id).collect())
    }

    async fn get_status(&self, run_id: &str) -> AppResult<Option<EngineStatusDto>> {
        let row = self
            .app
            .persist
            .get_execution(run_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(e) = row {
            let finished = matches!(e.status.as_str(), "COMPLETED" | "FAILED" | "TERMINATED");
            Ok(Some(EngineStatusDto {
                run_id: e.run_id,
                current_state: e.current_state_name.unwrap_or_else(|| "unknown".into()),
                last_task_state: None,
                status: e.status,
                parent_run_id: e.parent_run_id,
                parent_state_name: e.parent_state_name,
                context: e.context_snapshot.unwrap_or_else(|| serde_json::json!({})),
                updated_at: e.close_time.unwrap_or(e.start_time).and_utc(),
                finished,
            }))
        } else {
            Ok(None)
        }
    }

    async fn cleanup(&self, req: ControlRequest) -> AppResult<()> {
        let mut engines = self.app.engines.lock().await;
        engines.remove(&req.run_id);
        Ok(())
    }
}
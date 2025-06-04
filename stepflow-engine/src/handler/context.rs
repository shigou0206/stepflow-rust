use serde_json::Value;
use std::sync::Arc;
use chrono::Utc;
use stepflow_hook::{EngineEvent, EngineEventDispatcher};
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_sqlite::models::workflow_state::{WorkflowState, UpdateWorkflowState};
use crate::engine::WorkflowMode;

/// 统一的状态执行结果
#[derive(Debug)]
pub struct StateExecutionResult {
    pub output: Value,
    pub next_state: Option<String>,
    pub should_continue: bool,
}

/// 统一的状态执行上下文
pub struct StateExecutionContext<'a> {
    pub run_id: &'a str,
    pub state_name: &'a str,
    pub mode: WorkflowMode,
    pub dispatcher: &'a Arc<EngineEventDispatcher>,
    pub persistence: &'a Arc<dyn PersistenceManager>,
}

impl<'a> StateExecutionContext<'a> {
    pub fn new(
        run_id: &'a str,
        state_name: &'a str,
        mode: WorkflowMode,
        dispatcher: &'a Arc<EngineEventDispatcher>,
        persistence: &'a Arc<dyn PersistenceManager>,
    ) -> Self {
        Self {
            run_id,
            state_name,
            mode,
            dispatcher,
            persistence,
        }
    }

    pub async fn create_state_record(&self, input: &Value) -> Result<(), String> {
        let state = WorkflowState {
            state_id: format!("{}:{}", self.run_id, self.state_name),
            run_id: self.run_id.to_string(),
            state_name: self.state_name.to_string(),
            state_type: "state".to_string(),
            status: "started".to_string(),
            input: Some(input.to_string()),
            output: None,
            error: None,
            error_details: None,
            started_at: Some(Utc::now().naive_utc()),
            completed_at: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            version: 1,
            shard_id: 1,
        };
        
        self.persistence.create_state(&state)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update_success_state(&self, output: &Value) -> Result<(), String> {
        let update = UpdateWorkflowState {
            status: Some("succeeded".to_string()),
            output: Some(Some(output.to_string())),
            completed_at: Some(Some(Utc::now().naive_utc())),
            version: Some(2),
            ..Default::default()
        };
        
        self.persistence.update_state(
            &format!("{}:{}", self.run_id, self.state_name),
            &update
        ).await.map_err(|e| e.to_string())
    }

    pub async fn update_failure_state(&self, error: &str) -> Result<(), String> {
        let update = UpdateWorkflowState {
            status: Some("failed".to_string()),
            error: Some(Some(error.to_string())),
            completed_at: Some(Some(Utc::now().naive_utc())),
            version: Some(2),
            ..Default::default()
        };
        
        self.persistence.update_state(
            &format!("{}:{}", self.run_id, self.state_name),
            &update
        ).await.map_err(|e| e.to_string())
    }

    pub async fn dispatch_enter(&self, input: &Value) {
        self.dispatcher.dispatch(EngineEvent::NodeEnter {
            run_id: self.run_id.to_string(),
            state_name: self.state_name.to_string(),
            input: input.clone(),
        }).await;
    }

    pub async fn dispatch_success(&self, output: &Value) {
        self.dispatcher.dispatch(EngineEvent::NodeSuccess {
            run_id: self.run_id.to_string(),
            state_name: self.state_name.to_string(),
            output: output.clone(),
        }).await;
    }

    pub async fn dispatch_failure(&self, error: &str) {
        self.dispatcher.dispatch(EngineEvent::NodeFailed {
            run_id: self.run_id.to_string(),
            state_name: self.state_name.to_string(),
            error: error.to_string(),
        }).await;
    }
} 
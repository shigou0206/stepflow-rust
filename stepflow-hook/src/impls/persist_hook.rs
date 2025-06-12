use crate::EngineEventHandler;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_storage::traits::{WorkflowStorage, StateStorage, EventStorage};
use stepflow_storage::entities::{
    workflow_execution::{StoredWorkflowExecution, UpdateStoredWorkflowExecution},
    workflow_state::{StoredWorkflowState, UpdateStoredWorkflowState},
    workflow_event::StoredWorkflowEvent,
};
use chrono::Utc;
use std::sync::Arc;
use serde_json::json;

pub struct PersistHook {
    workflow: Arc<dyn WorkflowStorage>,
    state: Arc<dyn StateStorage>,
    event: Arc<dyn EventStorage>,
}

impl PersistHook {
    pub fn new(
        workflow: Arc<dyn WorkflowStorage>,
        state: Arc<dyn StateStorage>,
        event: Arc<dyn EventStorage>,
    ) -> Arc<Self> {
        Arc::new(Self { workflow, state, event })
    }

    async fn record_event(&self, run_id: &str, event_type: &str, message: &str) {
        let evt = StoredWorkflowEvent {
            id: 0,
            run_id: run_id.to_string(),
            shard_id: 1,
            event_id: 1,
            event_type: event_type.to_string(),
            state_id: None,
            state_type: None,
            trace_id: None,
            parent_event_id: None,
            context_version: None,
            attributes: Some(message.to_string()),
            attr_version: 1,
            timestamp: Utc::now().naive_utc(),
            archived: false,
        };
        let _ = self.event.create_event(&evt).await;
    }
}

#[async_trait::async_trait]
impl EngineEventHandler for PersistHook {
    async fn handle_event(&self, event: EngineEvent) {
        match event {
            EngineEvent::WorkflowStarted { run_id } => {
                let exec = StoredWorkflowExecution {
                    run_id: run_id.clone(),
                    workflow_id: Some(format!("wf_{}", run_id)),
                    shard_id: 1,
                    template_id: None,
                    mode: "default".to_string(),
                    current_state_name: None,
                    workflow_type: "default".to_string(),
                    status: "running".to_string(),
                    input: None,
                    input_version: 1,
                    result: None,
                    result_version: 1,
                    start_time: Utc::now().naive_utc(),
                    close_time: None,
                    current_event_id: 0,
                    memo: None,
                    search_attrs: None,
                    context_snapshot: None,
                    version: 1,
                };
                let _ = self.workflow.create_execution(&exec).await;
            }

            EngineEvent::NodeEnter { run_id, state_name, input } => {
                let state = StoredWorkflowState {
                    state_id: format!("{run_id}:{state_name}"),
                    run_id: run_id.clone(),
                    shard_id: 1,
                    state_name: state_name.clone(),
                    state_type: "default".to_string(),
                    input: Some(json!(input)),
                    status: "started".to_string(),
                    started_at: Some(Utc::now().naive_utc()),
                    output: None,
                    completed_at: None,
                    error: None,
                    error_details: None,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                    version: 1,
                };
                let _ = self.state.create_state(&state).await;
                self.record_event(&run_id, "NodeEnter", &format!("Entered state: {}", state_name)).await;
            }

            EngineEvent::NodeSuccess { run_id, state_name, output } => {
                let state_id = format!("{run_id}:{state_name}");
                let update = UpdateStoredWorkflowState {
                    state_name: None,
                    state_type: None,
                    input: None,
                    status: Some("succeeded".to_string()),
                    output: Some(Some(json!(output))),
                    completed_at: Some(Some(Utc::now().naive_utc())),
                    error: None,
                    error_details: None,
                    started_at: None,
                    version: None,
                };
                let _ = self.state.update_state(&state_id, &update).await;
            }

            EngineEvent::NodeFailed { run_id, state_name, error } => {
                let state_id = format!("{run_id}:{state_name}");
                let update = UpdateStoredWorkflowState {
                    state_name: None,
                    state_type: None,
                    input: None,
                    status: Some("failed".to_string()),
                    output: None,
                    completed_at: Some(Some(Utc::now().naive_utc())),
                    error: Some(Some(error)),
                    error_details: None,
                    started_at: None,
                    version: None,
                };
                let _ = self.state.update_state(&state_id, &update).await;
            }

            EngineEvent::WorkflowFinished { run_id, result } => {
                let update = UpdateStoredWorkflowExecution {
                    workflow_id: None,
                    shard_id: None,
                    template_id: None,
                    mode: None,
                    current_state_name: None,
                    workflow_type: None,
                    status: Some("completed".to_string()),
                    input: None,
                    input_version: None,
                    result: Some(Some(json!(result))),
                    result_version: None,
                    start_time: None,
                    close_time: Some(Some(Utc::now().naive_utc())),
                    current_event_id: None,
                    memo: None,
                    search_attrs: None,
                    context_snapshot: None,
                    version: None,
                };
                let _ = self.workflow.update_execution(&run_id, &update).await;
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use stepflow_storage::traits::{WorkflowStorage, StateStorage, EventStorage};
    use stepflow_storage::entities::{
        workflow_execution::{StoredWorkflowExecution, UpdateStoredWorkflowExecution},
        workflow_state::{StoredWorkflowState, UpdateStoredWorkflowState},
        workflow_event::StoredWorkflowEvent,
    };
    use stepflow_storage::error::StorageError;

    #[derive(Default)]
    struct MockPersistence {
        pub created_states: Arc<Mutex<Vec<StoredWorkflowState>>>,
        pub created_events: Arc<Mutex<Vec<StoredWorkflowEvent>>>,
        pub created_executions: Arc<Mutex<Vec<StoredWorkflowExecution>>>,
        pub updated_states: Arc<Mutex<Vec<(String, UpdateStoredWorkflowState)>>>,
        pub updated_executions: Arc<Mutex<Vec<(String, UpdateStoredWorkflowExecution)>>>,
    }

    #[async_trait]
    impl StateStorage for MockPersistence {
        async fn create_state(&self, state: &StoredWorkflowState) -> Result<(), StorageError> {
            self.created_states.lock().unwrap().push(state.clone());
            Ok(())
        }
        async fn get_state(&self, _: &str) -> Result<Option<StoredWorkflowState>, StorageError> { Ok(None) }
        async fn find_states_by_run_id(&self, _: &str, _: i64, _: i64) -> Result<Vec<StoredWorkflowState>, StorageError> { Ok(vec![]) }
        async fn update_state(&self, state_id: &str, update: &UpdateStoredWorkflowState) -> Result<(), StorageError> {
            self.updated_states.lock().unwrap().push((state_id.to_string(), update.clone()));
            Ok(())
        }
        async fn delete_state(&self, _: &str) -> Result<(), StorageError> { Ok(()) }
    }
    #[async_trait]
    impl EventStorage for MockPersistence {
        async fn create_event(&self, event: &StoredWorkflowEvent) -> Result<i64, StorageError> {
            self.created_events.lock().unwrap().push(event.clone());
            Ok(1)
        }
        async fn get_event(&self, _: i64) -> Result<Option<StoredWorkflowEvent>, StorageError> { Ok(None) }
        async fn find_events_by_run_id(&self, _: &str, _: i64, _: i64) -> Result<Vec<StoredWorkflowEvent>, StorageError> { Ok(vec![]) }
        async fn update_event(&self, _: i64, _: &stepflow_storage::entities::workflow_event::UpdateStoredWorkflowEvent) -> Result<(), StorageError> { Ok(()) }
        async fn archive_event(&self, _: i64) -> Result<(), StorageError> { Ok(()) }
        async fn delete_event(&self, _: i64) -> Result<(), StorageError> { Ok(()) }
        async fn delete_events_by_run_id(&self, _: &str) -> Result<u64, StorageError> { Ok(0) }
    }
    #[async_trait]
    impl WorkflowStorage for MockPersistence {
        async fn create_execution(&self, exec: &StoredWorkflowExecution) -> Result<(), StorageError> {
            self.created_executions.lock().unwrap().push(exec.clone());
            Ok(())
        }
        async fn get_execution(&self, _: &str) -> Result<Option<StoredWorkflowExecution>, StorageError> { Ok(None) }
        async fn find_executions(&self, _: i64, _: i64) -> Result<Vec<StoredWorkflowExecution>, StorageError> { Ok(vec![]) }
        async fn find_executions_by_status(&self, _: &str, _: i64, _: i64) -> Result<Vec<StoredWorkflowExecution>, StorageError> { Ok(vec![]) }
        async fn update_execution(&self, run_id: &str, update: &UpdateStoredWorkflowExecution) -> Result<(), StorageError> {
            self.updated_executions.lock().unwrap().push((run_id.to_string(), update.clone()));
            Ok(())
        }
        async fn delete_execution(&self, _: &str) -> Result<(), StorageError> { Ok(()) }
    }

    #[tokio::test]
    async fn test_node_enter_creates_state_and_event() {
        let mock = Arc::new(MockPersistence::default());
        let hook = PersistHook::new(mock.clone(), mock.clone(), mock.clone());

        let run_id = "test_run";
        let state_name = "MyState";
        let input = json!({ "key": "value" });

        hook.handle_event(EngineEvent::NodeEnter {
            run_id: run_id.to_string(),
            state_name: state_name.to_string(),
            input: input.clone(),
        }).await;

        // 验证 state 被创建
        let states = mock.created_states.lock().unwrap();
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].run_id, run_id);
        assert_eq!(states[0].state_name, state_name);
        assert_eq!(states[0].input, Some(input));

        // 验证事件被写入
        let events = mock.created_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].run_id, run_id);
        assert_eq!(events[0].event_type, "NodeEnter");
        assert!(events[0].attributes.as_ref().unwrap().contains(state_name));
    }
}
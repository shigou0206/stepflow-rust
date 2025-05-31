use crate::{EngineEvent, EngineEventHandler};
use stepflow_sqlite::models::workflow_execution::WorkflowExecution;
use stepflow_sqlite::models::workflow_state::WorkflowState;
use stepflow_sqlite::models::workflow_event::WorkflowEvent;
use stepflow_storage::PersistenceManager;
use chrono::Utc;
use std::sync::Arc;

pub struct PersistHook {
    persistence: Arc<dyn PersistenceManager>,
}

impl PersistHook {
    pub fn new(persistence: Arc<dyn PersistenceManager>) -> Arc<Self> {
        Arc::new(Self { persistence })
    }

    async fn record_event(&self, run_id: &str, event_type: &str, message: &str) {
        let evt = WorkflowEvent {
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
        let _ = self.persistence.create_event(&evt).await;
    }
}

#[async_trait::async_trait]
impl EngineEventHandler for PersistHook {
    async fn handle_event(&self, event: EngineEvent) {
        match event {
            EngineEvent::WorkflowStarted { run_id } => {
                let exec = WorkflowExecution {
                    run_id: run_id.clone(),
                    status: "running".into(),
                    start_time: Utc::now().naive_utc(),
                    ..Default::default()
                };
                let _ = self.persistence.create_execution(&exec).await;
            }

            EngineEvent::NodeEnter { run_id, state_name, input } => {
                let state = WorkflowState {
                    state_id: format!("{run_id}:{state_name}"),
                    run_id: run_id.clone(),
                    state_name: state_name.clone(),
                    input: Some(input.to_string()),
                    status: "started".into(),
                    started_at: Some(Utc::now().naive_utc()),
                    ..Default::default()
                };
                let _ = self.persistence.create_state(&state).await;
                self.record_event(&run_id, "NodeEnter", &format!("Entered state: {}", state_name)).await;
            }

            EngineEvent::NodeSuccess { run_id, state_name, output } => {
                let state_id = format!("{run_id}:{state_name}");
                let update = stepflow_sqlite::models::workflow_state::UpdateWorkflowState {
                    status: Some("succeeded".into()),
                    output: Some(output.to_string()),
                    completed_at: Some(Utc::now().naive_utc()),
                    ..Default::default()
                };
                let _ = self.persistence.update_state(&state_id, &update).await;
            }

            EngineEvent::NodeFailed { run_id, state_name, error } => {
                let state_id = format!("{run_id}:{state_name}");
                let update = stepflow_sqlite::models::workflow_state::UpdateWorkflowState {
                    status: Some("failed".into()),
                    error: Some(error),
                    completed_at: Some(Utc::now().naive_utc()),
                    ..Default::default()
                };
                let _ = self.persistence.update_state(&state_id, &update).await;
            }

            EngineEvent::WorkflowFinished { run_id, result } => {
                let update = stepflow_sqlite::models::workflow_execution::UpdateWorkflowExecution {
                    status: Some("completed".into()),
                    result: Some(result.to_string()),
                    close_time: Some(Utc::now().naive_utc()),
                    ..Default::default()
                };
                let _ = self.persistence.update_execution(&run_id, &update).await;
            }

            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    use stepflow_sqlite::models::{
        activity_task::*, queue_task::*, timer::*, workflow_execution::*,
        workflow_state::*, workflow_template::*, workflow_visibility::*,
    };
    use stepflow_storage::PersistenceManager;

    #[derive(Default)]
    struct MockPersistenceManager {
        pub created_states: Arc<Mutex<Vec<WorkflowState>>>,
        pub created_events: Arc<Mutex<Vec<WorkflowEvent>>>,
    }

    #[async_trait]
    impl PersistenceManager for MockPersistenceManager {
        async fn create_state(&self, state: &WorkflowState) -> Result<(), sqlx::Error> {
            self.created_states.lock().unwrap().push(state.clone());
            Ok(())
        }

        async fn create_event(&self, event: &WorkflowEvent) -> Result<i64, sqlx::Error> {
            self.created_events.lock().unwrap().push(event.clone());
            Ok(1)
        }

        async fn get_event(&self, _: i64) -> Result<Option<WorkflowEvent>, sqlx::Error> {
            Ok(None)
        }

        async fn find_events_by_run_id(&self, _: &str, _: i64, _: i64) -> Result<Vec<WorkflowEvent>, sqlx::Error> {
            Ok(vec![])
        }

        async fn archive_event(&self, _: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn delete_event(&self, _: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn delete_events_by_run_id(&self, _: &str) -> Result<u64, sqlx::Error> {
            Ok(0)
        }

        // 剩下接口用空实现填充即可
        async fn create_execution(&self, _: &WorkflowExecution) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn get_execution(&self, _: &str) -> Result<Option<WorkflowExecution>, sqlx::Error> {
            Ok(None)
        }

        async fn find_executions(&self, _: i64, _: i64) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
            Ok(vec![])
        }

        async fn find_executions_by_status(
            &self,
            _: &str,
            _: i64,
            _: i64,
        ) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
            Ok(vec![])
        }

        async fn update_execution(
            &self,
            _: &str,
            _: &UpdateWorkflowExecution,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn delete_execution(&self, _: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn get_state(&self, _: &str) -> Result<Option<WorkflowState>, sqlx::Error> {
            Ok(None)
        }

        async fn find_states_by_run_id(
            &self,
            _: &str,
            _: i64,
            _: i64,
        ) -> Result<Vec<WorkflowState>, sqlx::Error> {
            Ok(vec![])
        }

        async fn update_state(&self, _: &str, _: &UpdateWorkflowState) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn delete_state(&self, _: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }

        // 其他接口全部空实现
        async fn create_task(&self, _: &ActivityTask) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn get_task(&self, _: &str) -> Result<Option<ActivityTask>, sqlx::Error> {
            Ok(None)
        }
        async fn find_tasks_by_status(&self, _: &str, _: i64, _: i64) -> Result<Vec<ActivityTask>, sqlx::Error> {
            Ok(vec![])
        }
        async fn update_task(&self, _: &str, _: &UpdateActivityTask) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn delete_task(&self, _: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn create_timer(&self, _: &Timer) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn get_timer(&self, _: &str) -> Result<Option<Timer>, sqlx::Error> {
            Ok(None)
        }
        async fn update_timer(&self, _: &str, _: &UpdateTimer) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn delete_timer(&self, _: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn find_timers_before(
            &self,
            _: NaiveDateTime,
            _: i64,
        ) -> Result<Vec<Timer>, sqlx::Error> {
            Ok(vec![])
        }

        async fn create_template(&self, _: &WorkflowTemplate) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn get_template(&self, _: &str) -> Result<Option<WorkflowTemplate>, sqlx::Error> {
            Ok(None)
        }
        async fn find_templates(&self, _: i64, _: i64) -> Result<Vec<WorkflowTemplate>, sqlx::Error> {
            Ok(vec![])
        }
        async fn update_template(&self, _: &str, _: &UpdateWorkflowTemplate) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn delete_template(&self, _: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn create_visibility(&self, _: &WorkflowVisibility) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn get_visibility(&self, _: &str) -> Result<Option<WorkflowVisibility>, sqlx::Error> {
            Ok(None)
        }
        async fn find_visibilities_by_status(
            &self,
            _: &str,
            _: i64,
            _: i64,
        ) -> Result<Vec<WorkflowVisibility>, sqlx::Error> {
            Ok(vec![])
        }
        async fn update_visibility(
            &self,
            _: &str,
            _: &UpdateWorkflowVisibility,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn delete_visibility(&self, _: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn create_queue_task(&self, _: &QueueTask) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn get_queue_task(&self, _: &str) -> Result<Option<QueueTask>, sqlx::Error> {
            Ok(None)
        }
        async fn update_queue_task(&self, _: &str, _: &UpdateQueueTask) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn delete_queue_task(&self, _: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }
        async fn find_queue_tasks_by_status(
            &self,
            _: &str,
            _: i64,
            _: i64,
        ) -> Result<Vec<QueueTask>, sqlx::Error> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_node_enter_creates_state_and_event() {
        let mock = Arc::new(MockPersistenceManager::default());
        let hook = PersistHook::new(mock.clone());

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
        assert_eq!(states[0].input.as_deref(), Some(input.to_string().as_str()));

        // 验证事件被写入
        let events = mock.created_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].run_id, run_id);
        assert_eq!(events[0].event_type, "NodeEnter");
        assert!(events[0].attributes.as_ref().unwrap().contains(state_name));
    }
}
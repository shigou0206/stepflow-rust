use chrono::NaiveDateTime;

use stepflow_sqlite::models::workflow_execution::{WorkflowExecution, UpdateWorkflowExecution};
use stepflow_sqlite::models::workflow_event::WorkflowEvent;
use stepflow_sqlite::models::activity_task::{ActivityTask, UpdateActivityTask};
use stepflow_sqlite::models::workflow_state::{WorkflowState, UpdateWorkflowState};
use stepflow_sqlite::models::timer::{Timer, UpdateTimer};
use stepflow_sqlite::models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate};
use stepflow_sqlite::models::workflow_visibility::{WorkflowVisibility, UpdateWorkflowVisibility};

#[async_trait::async_trait]
pub trait PersistenceManager: Send + Sync {
    // workflow_execution接口定义：
    async fn create_execution(&self, exec: &WorkflowExecution) -> Result<(), sqlx::Error>;
    async fn get_execution(&self, run_id: &str) -> Result<Option<WorkflowExecution>, sqlx::Error>;
    async fn find_executions(&self, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>, sqlx::Error>;
    async fn find_executions_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>, sqlx::Error>;
    async fn update_execution(&self, run_id: &str, changes: &UpdateWorkflowExecution) -> Result<(), sqlx::Error>;
    async fn delete_execution(&self, run_id: &str) -> Result<(), sqlx::Error>;

    // workflow_event接口定义：
    async fn create_event(&self, event: &WorkflowEvent) -> Result<i64, sqlx::Error>;
    async fn get_event(&self, id: i64) -> Result<Option<WorkflowEvent>, sqlx::Error>;
    async fn find_events_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowEvent>, sqlx::Error>;
    async fn archive_event(&self, id: i64) -> Result<(), sqlx::Error>;
    async fn delete_event(&self, id: i64) -> Result<(), sqlx::Error>;
    async fn delete_events_by_run_id(&self, run_id: &str) -> Result<u64, sqlx::Error>;

    // 新增 activity_task 接口：
    async fn create_task(&self, task: &ActivityTask) -> Result<(), sqlx::Error>;
    async fn get_task(&self, task_token: &str) -> Result<Option<ActivityTask>, sqlx::Error>;
    async fn find_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<ActivityTask>, sqlx::Error>;
    async fn update_task(&self, task_token: &str, changes: &UpdateActivityTask) -> Result<(), sqlx::Error>;
    async fn delete_task(&self, task_token: &str) -> Result<(), sqlx::Error>;

    // workflow_state 接口：
    async fn create_state(&self, state: &WorkflowState) -> Result<(), sqlx::Error>;
    async fn get_state(&self, state_id: &str) -> Result<Option<WorkflowState>, sqlx::Error>;
    async fn find_states_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowState>, sqlx::Error>;
    async fn update_state(&self, state_id: &str, changes: &UpdateWorkflowState) -> Result<(), sqlx::Error>;
    async fn delete_state(&self, state_id: &str) -> Result<(), sqlx::Error>;

    // timer 接口定义：
    async fn create_timer(&self, timer: &Timer) -> Result<(), sqlx::Error>;
    async fn get_timer(&self, timer_id: &str) -> Result<Option<Timer>, sqlx::Error>;
    async fn update_timer(&self, timer_id: &str, changes: &UpdateTimer) -> Result<(), sqlx::Error>;
    async fn delete_timer(&self, timer_id: &str) -> Result<(), sqlx::Error>;
    async fn find_timers_before(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<Timer>, sqlx::Error>;

    // workflow_template 接口定义：
    async fn create_template(&self, tpl: &WorkflowTemplate) -> Result<(), sqlx::Error>;
    async fn get_template(&self, template_id: &str) -> Result<Option<WorkflowTemplate>, sqlx::Error>;
    async fn find_templates(&self, limit: i64, offset: i64) -> Result<Vec<WorkflowTemplate>, sqlx::Error>;
    async fn update_template(&self, template_id: &str, changes: &UpdateWorkflowTemplate) -> Result<(), sqlx::Error>;
    async fn delete_template(&self, template_id: &str) -> Result<(), sqlx::Error>;

    // workflow_visibility 接口定义
    async fn create_visibility(&self, vis: &WorkflowVisibility) -> Result<(), sqlx::Error>;
    async fn get_visibility(&self, run_id: &str) -> Result<Option<WorkflowVisibility>, sqlx::Error>;
    async fn find_visibilities_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowVisibility>, sqlx::Error>;
    async fn update_visibility(&self, run_id: &str, changes: &UpdateWorkflowVisibility) -> Result<(), sqlx::Error>;
    async fn delete_visibility(&self, run_id: &str) -> Result<(), sqlx::Error>;
}
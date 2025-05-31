use sqlx::SqlitePool;
use chrono::NaiveDateTime;
use crate::{
    PersistenceManager,
    persistence::{
        workflow_execution::WorkflowExecutionPersistence,
        workflow_event::WorkflowEventPersistence,
        activity_task::ActivityTaskPersistence,
        workflow_state::WorkflowStatePersistence,
        timer::TimerPersistence,
        workflow_template::WorkflowTemplatePersistence,
        workflow_visibility::WorkflowVisibilityPersistence,
    },
};
use stepflow_sqlite::models::workflow_execution::{WorkflowExecution, UpdateWorkflowExecution};
use stepflow_sqlite::models::workflow_event::WorkflowEvent;
use stepflow_sqlite::models::activity_task::{ActivityTask, UpdateActivityTask};
use stepflow_sqlite::models::workflow_state::{WorkflowState, UpdateWorkflowState};
use stepflow_sqlite::models::timer::{Timer, UpdateTimer};
use stepflow_sqlite::models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate};
use stepflow_sqlite::models::workflow_visibility::{WorkflowVisibility, UpdateWorkflowVisibility};
#[derive(Clone)]
pub struct PersistenceManagerImpl {
    execution: WorkflowExecutionPersistence,
    event: WorkflowEventPersistence,
    activity_task: ActivityTaskPersistence,
    workflow_state: WorkflowStatePersistence,
    timer: TimerPersistence,
    workflow_template: WorkflowTemplatePersistence,
    workflow_visibility: WorkflowVisibilityPersistence,
}

impl PersistenceManagerImpl {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            execution: WorkflowExecutionPersistence::new(pool.clone()),
            event: WorkflowEventPersistence::new(pool.clone()),
            activity_task: ActivityTaskPersistence::new(pool.clone()),
            workflow_state: WorkflowStatePersistence::new(pool.clone()),
            timer: TimerPersistence::new(pool.clone()),
            workflow_template: WorkflowTemplatePersistence::new(pool.clone()),
            workflow_visibility: WorkflowVisibilityPersistence::new(pool.clone()),
        }
    }
}

#[async_trait::async_trait]
impl PersistenceManager for PersistenceManagerImpl {
    async fn create_execution(&self, exec: &WorkflowExecution) -> Result<(), sqlx::Error> {
        self.execution.create(exec).await
    }

    async fn get_execution(&self, run_id: &str) -> Result<Option<WorkflowExecution>, sqlx::Error> {
        self.execution.get(run_id).await
    }

    async fn find_executions(&self, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
        self.execution.find(limit, offset).await
    }

    async fn find_executions_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
        self.execution.find_by_status(status, limit, offset).await
    }

    async fn update_execution(
        &self,
        run_id: &str,
        changes: &UpdateWorkflowExecution,
    ) -> Result<(), sqlx::Error> {
        self.execution.update(run_id, changes).await
    }

    async fn delete_execution(&self, run_id: &str) -> Result<(), sqlx::Error> {
        self.execution.delete(run_id).await
    }

    async fn create_event(&self, event: &WorkflowEvent) -> Result<i64, sqlx::Error> {
        self.event.create_event(event).await
    }

    async fn get_event(&self, id: i64) -> Result<Option<WorkflowEvent>, sqlx::Error> {
        self.event.get_event(id).await
    }

    async fn find_events_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowEvent>, sqlx::Error> {
        self.event.find_events_by_run_id(run_id, limit, offset).await
    }

    async fn archive_event(&self, id: i64) -> Result<(), sqlx::Error> {
        self.event.archive_event(id).await
    }

    async fn delete_event(&self, id: i64) -> Result<(), sqlx::Error> {
        self.event.delete_event(id).await
    }

    async fn delete_events_by_run_id(&self, run_id: &str) -> Result<u64, sqlx::Error> {
        self.event.delete_events_by_run_id(run_id).await
    }

    // activity_task 实现
    async fn create_task(&self, task: &ActivityTask) -> Result<(), sqlx::Error> {
        self.activity_task.create_task(task).await
    }

    async fn get_task(&self, task_token: &str) -> Result<Option<ActivityTask>, sqlx::Error> {
        self.activity_task.get_task(task_token).await
    }

    async fn find_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<ActivityTask>, sqlx::Error> {
        self.activity_task.find_tasks_by_status(status, limit, offset).await
    }

    async fn update_task(&self, task_token: &str, changes: &UpdateActivityTask) -> Result<(), sqlx::Error> {
        self.activity_task.update_task(task_token, changes).await
    }

    async fn delete_task(&self, task_token: &str) -> Result<(), sqlx::Error> {
        self.activity_task.delete_task(task_token).await
    }

    // workflow_state实现接口
    async fn create_state(&self, state: &WorkflowState) -> Result<(), sqlx::Error> {
        self.workflow_state.create_state(state).await
    }

    async fn get_state(&self, state_id: &str) -> Result<Option<WorkflowState>, sqlx::Error> {
        self.workflow_state.get_state(state_id).await
    }

    async fn find_states_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowState>, sqlx::Error> {
        self.workflow_state.find_states_by_run_id(run_id, limit, offset).await
    }

    async fn update_state(&self, state_id: &str, changes: &UpdateWorkflowState) -> Result<(), sqlx::Error> {
        self.workflow_state.update_state(state_id, changes).await
    }

    async fn delete_state(&self, state_id: &str) -> Result<(), sqlx::Error> {
        self.workflow_state.delete_state(state_id).await
    }

    // Timer实现接口
    async fn create_timer(&self, timer: &Timer) -> Result<(), sqlx::Error> {
        self.timer.create_timer(timer).await
    }

    async fn get_timer(&self, timer_id: &str) -> Result<Option<Timer>, sqlx::Error> {
        self.timer.get_timer(timer_id).await
    }

    async fn update_timer(&self, timer_id: &str, changes: &UpdateTimer) -> Result<(), sqlx::Error> {
        self.timer.update_timer(timer_id, changes).await
    }

    async fn delete_timer(&self, timer_id: &str) -> Result<(), sqlx::Error> {
        self.timer.delete_timer(timer_id).await
    }

    async fn find_timers_before(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<Timer>, sqlx::Error> {
        self.timer.find_timers_before(before, limit).await
    }

    // workflow_template实现接口
    async fn create_template(&self, tpl: &WorkflowTemplate) -> Result<(), sqlx::Error> {
        self.workflow_template.create_template(tpl).await
    }

    async fn get_template(&self, template_id: &str) -> Result<Option<WorkflowTemplate>, sqlx::Error> {
        self.workflow_template.get_template(template_id).await
    }

    async fn find_templates(&self, limit: i64, offset: i64) -> Result<Vec<WorkflowTemplate>, sqlx::Error> {
        self.workflow_template.find_templates(limit, offset).await
    }

    async fn update_template(&self, template_id: &str, changes: &UpdateWorkflowTemplate) -> Result<(), sqlx::Error> {
        self.workflow_template.update_template(template_id, changes).await
    }

    async fn delete_template(&self, template_id: &str) -> Result<(), sqlx::Error> {
        self.workflow_template.delete_template(template_id).await
    }

    // workflow_visibility接口实现
    async fn create_visibility(&self, vis: &WorkflowVisibility) -> Result<(), sqlx::Error> {
        self.workflow_visibility.create_visibility(vis).await
    }

    async fn get_visibility(&self, run_id: &str) -> Result<Option<WorkflowVisibility>, sqlx::Error> {
        self.workflow_visibility.get_visibility(run_id).await
    }

    async fn find_visibilities_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowVisibility>, sqlx::Error> {
        self.workflow_visibility.find_visibilities_by_status(status, limit, offset).await
    }

    async fn update_visibility(&self, run_id: &str, changes: &UpdateWorkflowVisibility) -> Result<(), sqlx::Error> {
        self.workflow_visibility.update_visibility(run_id, changes).await
    }

    async fn delete_visibility(&self, run_id: &str) -> Result<(), sqlx::Error> {
        self.workflow_visibility.delete_visibility(run_id).await
    }
}
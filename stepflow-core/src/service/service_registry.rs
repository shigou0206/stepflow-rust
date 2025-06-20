use std::sync::Arc;
use crate::service::{
    TemplateService,
    ExecutionService,
    ActivityTaskService,
    WorkflowEventService,
    QueueTaskService,
    TimerService,
};

#[derive(Clone)]
pub struct ServiceRegistry {
    pub template: Arc<dyn TemplateService>,
    pub execution: Arc<dyn ExecutionService>,
    pub activity_task: Arc<dyn ActivityTaskService>,
    pub workflow_event: Arc<dyn WorkflowEventService>,
    pub queue_task: Arc<dyn QueueTaskService>,
    pub timer: Arc<dyn TimerService>,
}
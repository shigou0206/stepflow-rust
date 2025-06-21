use std::sync::Arc;
use crate::service::{
    TemplateService, 
    ExecutionService, 
    ActivityTaskService, 
    WorkflowEventService, 
    QueueTaskService,
    TimerService,
    WorkflowEngineService,
    DummyServiceImpl,
};

#[derive(Clone)]
pub struct ServiceRegistry {
    pub template: Arc<dyn TemplateService>,
    pub execution: Arc<dyn ExecutionService>,
    pub activity_task: Arc<dyn ActivityTaskService>,
    pub workflow_event: Arc<dyn WorkflowEventService>,
    pub queue_task: Arc<dyn QueueTaskService>,
    pub timer: Arc<dyn TimerService>,
    pub engine: Arc<dyn WorkflowEngineService>,
    // pub match_service: Arc<dyn MatchService>,
    // pub persist: DynPM,
}

impl ServiceRegistry {
    pub fn empty() -> Self {
        let dummy = Arc::new(DummyServiceImpl);
        Self {
            template: dummy.clone(),
            execution: dummy.clone(),
            activity_task: dummy.clone(),
            workflow_event: dummy.clone(),
            queue_task: dummy.clone(),
            timer: dummy.clone(),
            engine: dummy.clone(),
        }
    }
}
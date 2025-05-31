pub mod db;
pub mod schema;

pub mod models {
    pub mod activity_task;
    pub mod timer;
    pub mod workflow_event;
    pub mod workflow_execution;
    pub mod workflow_state;
    pub mod workflow_template;
    pub mod workflow_visibility;
    pub mod queue_task;
}

pub mod crud {
    pub mod workflow_template_crud;
    pub mod workflow_execution_crud;
    pub mod workflow_state_crud;
    pub mod activity_task_crud;
    pub mod timer_crud;
    pub mod workflow_event_crud;
    pub mod workflow_visibility_crud;
    pub mod queue_task_crud;
}

#[macro_export]
macro_rules! tx_exec {
    ($tx:expr, $fn:ident($($arg:expr),*)) => {
        $fn(&mut *$tx, $($arg),*).await
    };
}
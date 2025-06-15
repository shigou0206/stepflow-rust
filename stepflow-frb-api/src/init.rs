use once_cell::sync::OnceCell;
use std::sync::Arc;

use stepflow_common::config::StepflowConfig;
use stepflow_core::app_state::AppState;
use stepflow_core::builder::build_app_state;
use stepflow_gateway::service::execution::ExecutionSqlxSvc;
use stepflow_eventbus::core::bus::EventBus;

static EXECUTION_SVC: OnceCell<ExecutionSqlxSvc> = OnceCell::new();
static GLOBAL_EVENT_BUS: OnceCell<Arc<dyn EventBus>> = OnceCell::new();

pub async fn init_app_state(cfg: &StepflowConfig) {
    let app_state: AppState = build_app_state(cfg).await.expect("Failed to build AppState");

    let svc = ExecutionSqlxSvc::new(Arc::new(app_state.clone()));
    EXECUTION_SVC.set(svc).unwrap();

    GLOBAL_EVENT_BUS.set(app_state.event_bus.clone()).unwrap();
}

pub fn get_execution_svc() -> &'static ExecutionSqlxSvc {
    EXECUTION_SVC
        .get()
        .expect("ExecutionSqlxSvc not initialized")
}

pub fn get_event_bus() -> Arc<dyn EventBus> {
    GLOBAL_EVENT_BUS
        .get()
        .expect("EventBus not initialized")
        .clone()
}
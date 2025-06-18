pub use stepflow_gateway::service::execution::ExecutionSqlxSvc;

use flutter_rust_bridge::frb;
use crate::execution_api::*;
use crate::execution_types::*;
use crate::init::*;

#[frb]
pub async fn start_execution_request(req: FrbStartExecutionRequest) -> Result<FrbExecutionResult, String> {
    start_execution(get_execution_svc(), req).await
}

#[frb]
pub async fn get_execution_by_id(run_id: String) -> Result<FrbExecutionResult, String> {
    get_execution(get_execution_svc(), run_id).await
}

#[frb]
pub async fn list_executions_request(req: FrbListRequest) -> Result<Vec<FrbExecutionResult>, String> {
    list_executions(get_execution_svc(), req).await
}

#[frb]
pub async fn update_execution_request(run_id: String, req: FrbExecUpdateRequest) -> Result<(), String> {
    update_execution(get_execution_svc(), run_id, req).await
}

#[frb]
pub async fn delete_execution_request(run_id: String) -> Result<(), String> {
    delete_execution(get_execution_svc(), run_id).await
}

#[frb]
pub async fn list_executions_by_status_request(req: FrbListByStatusRequest) -> Result<Vec<FrbExecutionResult>, String> {
    list_executions_by_status(get_execution_svc(), req).await
}

fn get_execution_svc() -> &'static ExecutionSqlxSvc {
    EXECUTION_SVC
        .get()
        .expect("ExecutionSqlxSvc not initialized")
}
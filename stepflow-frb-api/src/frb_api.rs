use flutter_rust_bridge::frb;
use crate::execution_api::*;
use crate::execution_types::*;
use crate::init::get_execution_svc;

#[frb]
pub async fn startExecution(req: FrbStartExecutionRequest) -> Result<FrbExecutionResult, String> {
    start_execution(get_execution_svc(), req).await
}

#[frb]
pub async fn getExecution(run_id: String) -> Result<FrbExecutionResult, String> {
    get_execution(get_execution_svc(), run_id).await
}

#[frb]
pub async fn listExecutions(req: FrbListRequest) -> Result<Vec<FrbExecutionResult>, String> {
    list_executions(get_execution_svc(), req).await
}

#[frb]
pub async fn updateExecution(run_id: String, req: FrbExecUpdateRequest) -> Result<(), String> {
    update_execution(get_execution_svc(), run_id, req).await
}

#[frb]
pub async fn deleteExecution(run_id: String) -> Result<(), String> {
    delete_execution(get_execution_svc(), run_id).await
}

#[frb]
pub async fn listExecutionsByStatus(req: FrbListByStatusRequest) -> Result<Vec<FrbExecutionResult>, String> {
    list_executions_by_status(get_execution_svc(), req).await
}
use stepflow_gateway::service::execution::ExecutionSqlxSvc;

use crate::execution_types::*;
use stepflow_dto::dto::execution::ExecStart;
use stepflow_gateway::service::ExecutionService;
use serde_json::Value;

#[cfg(not(frb_expand))]
pub async fn start_execution(svc: &ExecutionSqlxSvc, req: FrbStartExecutionRequest) -> Result<FrbExecutionResult, String> {
    let dsl = match &req.dsl_json {
        Some(json) => Some(serde_json::from_str(json).map_err(|e| format!("invalid dsl_json: {e}"))?),
        None => None,
    };
    let init_ctx = match &req.init_ctx_json {
        Some(json) => Some(serde_json::from_str(json).map_err(|e| format!("invalid init_ctx_json: {e}"))?),
        None => None,
    };

    let inner = ExecStart {
        template_id: req.template_id.clone(),
        dsl,
        init_ctx,
        mode: req.mode.clone(),
    };

    let dto = svc.start(inner).await.map_err(|e| format!("{e}"))?;

    Ok(FrbExecutionResult {
        run_id: dto.run_id,
        mode: dto.mode,
        status: dto.status,
        result_json: dto.result.map(|v| serde_json::to_string(&v).unwrap()),
        started_at: dto.started_at.to_rfc3339(),
        finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
    })
}

#[cfg(not(frb_expand))]
pub async fn get_execution(svc: &ExecutionSqlxSvc, run_id: String) -> Result<FrbExecutionResult, String> {
    let dto = svc.get(&run_id).await.map_err(|e| format!("{e}"))?;
    Ok(FrbExecutionResult {
        run_id: dto.run_id,
        mode: dto.mode,
        status: dto.status,
        result_json: dto.result.map(|v| serde_json::to_string(&v).unwrap()),
        started_at: dto.started_at.to_rfc3339(),
        finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
    })
}

#[cfg(not(frb_expand))]
pub async fn list_executions(svc: &ExecutionSqlxSvc, req: FrbListRequest) -> Result<Vec<FrbExecutionResult>, String> {
    let list = svc
        .list(req.limit.unwrap_or(20), req.offset.unwrap_or(0))
        .await
        .map_err(|e| format!("{e}"))?;

    Ok(list
        .into_iter()
        .map(|dto| FrbExecutionResult {
            run_id: dto.run_id,
            mode: dto.mode,
            status: dto.status,
            result_json: dto.result.map(|v| serde_json::to_string(&v).unwrap()),
            started_at: dto.started_at.to_rfc3339(),
            finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
        })
        .collect())
}

#[cfg(not(frb_expand))]
pub async fn update_execution(svc: &ExecutionSqlxSvc, run_id: String, req: FrbExecUpdateRequest) -> Result<(), String> {
    let result = match &req.result_json {
        Some(json) => Some(serde_json::from_str::<Value>(json).map_err(|e| format!("invalid result_json: {e}"))?),
        None => None,
    };

    svc.update(&run_id, req.status.clone(), result)
        .await
        .map_err(|e| format!("{e}"))
}

#[cfg(not(frb_expand))]
pub async fn delete_execution(svc: &ExecutionSqlxSvc, run_id: String) -> Result<(), String> {
    svc.delete(&run_id).await.map_err(|e| format!("{e}"))
}

#[cfg(not(frb_expand))]
pub async fn list_executions_by_status(
    svc: &ExecutionSqlxSvc,
    req: FrbListByStatusRequest,
) -> Result<Vec<FrbExecutionResult>, String> {
    let list = svc
        .list_by_status(&req.status, req.limit.unwrap_or(20), req.offset.unwrap_or(0))
        .await
        .map_err(|e| format!("{e}"))?;

    Ok(list
        .into_iter()
        .map(|dto| FrbExecutionResult {
            run_id: dto.run_id,
            mode: dto.mode,
            status: dto.status,
            result_json: dto.result.map(|v| serde_json::to_string(&v).unwrap()),
            started_at: dto.started_at.to_rfc3339(),
            finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
        })
        .collect())
}
use crate::execution_types::*;
use stepflow_dto::dto::execution::ExecStart;
use stepflow_gateway::service::ExecutionService;
use stepflow_gateway::service::execution::ExecutionSqlxSvc;

pub async fn start_execution(svc: &ExecutionSqlxSvc, req: FrbStartExecutionRequest) -> Result<FrbExecutionResult, String> {
    let inner = ExecStart {
        template_id: req.template_id.clone(),
        dsl: req.dsl.clone(),
        init_ctx: req.init_ctx.clone(),
        mode: req.mode.clone(),
    };

    let dto = svc.start(inner).await.map_err(|e| format!("{e}"))?;

    Ok(FrbExecutionResult {
        run_id: dto.run_id,
        mode: dto.mode,
        status: dto.status,
        result: dto.result,
        started_at: dto.started_at.to_rfc3339(),
        finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
    })
}

pub async fn get_execution(svc: &ExecutionSqlxSvc, run_id: String) -> Result<FrbExecutionResult, String> {
    let dto = svc.get(&run_id).await.map_err(|e| format!("{e}"))?;
    Ok(FrbExecutionResult {
        run_id: dto.run_id,
        mode: dto.mode,
        status: dto.status,
        result: dto.result,
        started_at: dto.started_at.to_rfc3339(),
        finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
    })
}

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
            result: dto.result,
            started_at: dto.started_at.to_rfc3339(),
            finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
        })
        .collect())
}

pub async fn update_execution(svc: &ExecutionSqlxSvc, run_id: String, req: FrbExecUpdateRequest) -> Result<(), String> {
    svc.update(&run_id, req.status.clone(), req.result.clone())
        .await
        .map_err(|e| format!("{e}"))
}

pub async fn delete_execution(svc: &ExecutionSqlxSvc, run_id: String) -> Result<(), String> {
    svc.delete(&run_id).await.map_err(|e| format!("{e}"))
}

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
            result: dto.result,
            started_at: dto.started_at.to_rfc3339(),
            finished_at: dto.finished_at.map(|t| t.to_rfc3339()),
        })
        .collect())
}
use std::sync::Arc;
use axum::{
    routing::{get, post},
    Json, Router,
    extract::{Path, State, Query}
};
use stepflow_dto::dto::execution::*;
use stepflow_core::{
    app_state::AppState,
    error::AppResult,
    service::ExecutionService,
};
use serde_json::Value;

#[derive(serde::Deserialize)]
pub struct Page {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct StatusPage {
    pub status: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ExecUpdate {
    pub status: String,
    pub result: Option<Value>,
}

pub fn router(svc: Arc<dyn ExecutionService>) -> Router<AppState> {
    Router::new()
        .route("/", post(start).get(list))
        .route("/:id", get(get_one).put(update).delete(delete_one))
        .route("/by_status", get(list_by_status))
        .with_state(svc)
}

/// 启动工作流执行
#[utoipa::path(
    post,
    path = "/v1/executions",
    request_body = ExecStart,
    responses(
        (status = 200, description = "成功启动工作流执行", body = ExecDto),
        (status = 400, description = "请求参数错误"),
        (status = 404, description = "模板不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "executions"
)]
pub async fn start(
    State(svc): State<Arc<dyn ExecutionService>>,
    Json(body): Json<ExecStart>,
) -> AppResult<Json<ExecDto>> {
    Ok(Json(svc.start(body).await?))
}

/// 获取工作流执行列表
#[utoipa::path(
    get,
    path = "/v1/executions",
    params(
        ("limit" = Option<i64>, Query, description = "每页数量"),
        ("offset" = Option<i64>, Query, description = "偏移量")
    ),
    responses(
        (status = 200, description = "成功获取工作流执行列表", body = Vec<ExecDto>),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "executions"
)]
pub async fn list(
    State(svc): State<Arc<dyn ExecutionService>>,
    Query(p): Query<Page>,
) -> AppResult<Json<Vec<ExecDto>>> {
    Ok(Json(svc.list(p.limit.unwrap_or(20), p.offset.unwrap_or(0)).await?))
}

/// 获取工作流执行详情
#[utoipa::path(
    get,
    path = "/v1/executions/{id}",
    params(
        ("id" = String, Path, description = "执行 ID")
    ),
    responses(
        (status = 200, description = "成功获取工作流执行", body = ExecDto),
        (status = 404, description = "执行不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "executions"
)]
pub async fn get_one(
    State(svc): State<Arc<dyn ExecutionService>>,
    Path(id): Path<String>,
) -> AppResult<Json<ExecDto>> {
    Ok(Json(svc.get(&id).await?))
}

/// 更新执行状态
#[utoipa::path(
    put,
    path = "/v1/executions/{id}",
    request_body = ExecUpdate,
    params(
        ("id" = String, Path, description = "执行 ID")
    ),
    responses(
        (status = 200, description = "成功更新工作流状态", body = ExecDto),
        (status = 404, description = "执行不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "executions"
)]
pub async fn update(
    State(svc): State<Arc<dyn ExecutionService>>,
    Path(id): Path<String>,
    Json(body): Json<ExecUpdate>,
) -> AppResult<Json<ExecDto>> {
    svc.update(&id, body.status.clone(), body.result.clone()).await?;
    let dto = svc.get(&id).await?;
    Ok(Json(dto))
}

/// 删除执行记录
#[utoipa::path(
    delete,
    path = "/v1/executions/{id}",
    params(
        ("id" = String, Path, description = "执行 ID")
    ),
    responses(
        (status = 200, description = "成功删除执行记录"),
        (status = 404, description = "执行不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "executions"
)]
pub async fn delete_one(
    State(svc): State<Arc<dyn ExecutionService>>,
    Path(id): Path<String>,
) -> AppResult<()> {
    svc.delete(&id).await?;
    Ok(())
}

/// 按状态分页获取执行记录
#[utoipa::path(
    get,
    path = "/v1/executions/by_status",
    params(
        ("status" = String, Query, description = "状态"),
        ("limit" = Option<i64>, Query, description = "每页数量"),
        ("offset" = Option<i64>, Query, description = "偏移量")
    ),
    responses(
        (status = 200, description = "成功获取执行记录列表", body = Vec<ExecDto>),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "executions"
)]
pub async fn list_by_status(
    State(svc): State<Arc<dyn ExecutionService>>,
    Query(p): Query<StatusPage>,
) -> AppResult<Json<Vec<ExecDto>>> {
    Ok(Json(svc.list_by_status(
        &p.status,
        p.limit.unwrap_or(20),
        p.offset.unwrap_or(0),
    ).await?))
}
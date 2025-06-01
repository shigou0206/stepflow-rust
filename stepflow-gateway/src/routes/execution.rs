// gateway/src/routes/execution.rs
use axum::{
    routing::{get, post},
    Json, Router,
    extract::{Path, State, Query}
};
use crate::{
    dto::execution::{ExecStart, ExecDto},
    service::{ExecutionSvc, ExecutionService},
    error::AppResult,
    app_state::AppState,
};

#[derive(serde::Deserialize)]
pub struct Page {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn router(svc: ExecutionSvc) -> Router<AppState> {
    Router::new()
        .route("/", post(start).get(list))
        .route("/:id", get(get_one))
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
    State(svc): State<ExecutionSvc>,
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
    State(svc): State<ExecutionSvc>,
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
    State(svc): State<ExecutionSvc>,
    Path(id): Path<String>,
) -> AppResult<Json<ExecDto>> {
    Ok(Json(svc.get(&id).await?))
}
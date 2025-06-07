use axum::{
    routing::{get, post},
    Json, Router,
    extract::{Path, State}
};
use stepflow_dto::dto::template::{TemplateUpsert, TemplateDto};   

use crate::{
    service::{TemplateSvc, TemplateService},
    error::AppResult,
    app_state::AppState,
};

pub fn router(svc: TemplateSvc) -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/:id", get(get_one).put(update).delete(delete_one))
        .with_state(svc)
}

/// 创建工作流模板
#[utoipa::path(
    post,
    path = "/v1/templates",
    request_body = TemplateUpsert,
    responses(
        (status = 200, description = "成功创建工作流模板", body = TemplateDto),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "templates"
)]
pub async fn create(
    State(svc): State<TemplateSvc>,
    Json(body): Json<TemplateUpsert>,
) -> AppResult<Json<TemplateDto>> {
    Ok(Json(svc.create(body).await?))
}

/// 获取工作流模板列表
#[utoipa::path(
    get,
    path = "/v1/templates",
    responses(
        (status = 200, description = "成功获取工作流模板列表", body = Vec<TemplateDto>),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "templates"
)]
pub async fn list(
    State(svc): State<TemplateSvc>
) -> AppResult<Json<Vec<TemplateDto>>> {
    Ok(Json(svc.list().await?))
}

/// 获取工作流模板详情
#[utoipa::path(
    get,
    path = "/v1/templates/{id}",
    params(
        ("id" = String, Path, description = "模板 ID")
    ),
    responses(
        (status = 200, description = "成功获取工作流模板", body = TemplateDto),
        (status = 404, description = "模板不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "templates"
)]
pub async fn get_one(
    State(svc): State<TemplateSvc>,
    Path(id): Path<String>,
) -> AppResult<Json<TemplateDto>> {
    Ok(Json(svc.get(&id).await?))
}

/// 更新工作流模板
#[utoipa::path(
    put,
    path = "/v1/templates/{id}",
    params(
        ("id" = String, Path, description = "模板 ID")
    ),
    request_body = TemplateUpsert,
    responses(
        (status = 200, description = "成功更新工作流模板", body = TemplateDto),
        (status = 404, description = "模板不存在"),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "templates"
)]
pub async fn update(
    State(svc): State<TemplateSvc>,
    Path(id): Path<String>,
    Json(body): Json<TemplateUpsert>,
) -> AppResult<Json<TemplateDto>> {
    Ok(Json(svc.update(&id, body).await?))
}

/// 删除工作流模板
#[utoipa::path(
    delete,
    path = "/v1/templates/{id}",
    params(
        ("id" = String, Path, description = "模板 ID")
    ),
    responses(
        (status = 200, description = "成功删除工作流模板"),
        (status = 404, description = "模板不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "templates"
)]
pub async fn delete_one(
    State(svc): State<TemplateSvc>,
    Path(id): Path<String>,
) -> AppResult<()> {
    svc.delete(&id).await?;
    Ok(())
}
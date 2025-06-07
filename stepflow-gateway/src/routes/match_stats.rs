use axum::{routing::get, Json, Router, extract::State};
use crate::dto::match_stats::MatchStatsResponse;
use crate::service::MatchStatsService;
use crate::error::AppResult;
use std::sync::Arc;

/// 路由注册函数
pub fn router(svc: Arc<dyn MatchStatsService>) -> Router {
    Router::new()
        .route("/stats", get(get_match_stats))
        .with_state(svc)
}

/// 获取 match 队列统计
#[utoipa::path(
    get,
    path = "/v1/match/stats",
    responses(
        (status = 200, description = "Match 队列统计信息", body = MatchStatsResponse)
    ),
    tag = "match"
)]
pub async fn get_match_stats(
    State(svc): State<Arc<dyn MatchStatsService>>,
) -> AppResult<Json<MatchStatsResponse>> {
    Ok(Json(svc.collect_stats().await?))
}
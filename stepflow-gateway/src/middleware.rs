use http::{Method, header::{CONTENT_TYPE, HeaderValue}};
use tower::{ServiceBuilder, layer::util::Identity, Layer};
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
    compression::CompressionLayer,
};
use std::time::Duration;
use axum::Router;

/// 默认中间件栈配置
/// 
/// 包含:
/// - 请求追踪 (TraceLayer)
/// - CORS 配置
/// - 响应压缩
pub fn default_stack<S>(router: Router<S>) -> Router<S> 
where 
    S: Clone + Send + Sync + 'static,
{
    router
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
}

/// CORS 配置
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        // 允许的源
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        // 允许的方法
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ])
        // 允许的头
        .allow_headers([CONTENT_TYPE])
        // 允许携带认证信息
        .allow_credentials(true)
}
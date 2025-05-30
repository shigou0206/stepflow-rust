//! 全局统一错误类型

use thiserror::Error;

/// 每个 resolver / 引擎阶段都可直接返回 MappingError
#[derive(Debug, Error)]
pub enum MappingError {
    // ───────────────────── 基础序列化/反序列化 ─────────────────────
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("serde yaml error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    // ───────────────────── 解析 & 执行错误 ───────────────────────
    #[error("jsonpath error: {0}")]
    JsonPath(String),

    #[error("expression error: {0}")]
    Expr(String),

    #[error("template render error: {0}")]
    Template(String),

    // ───────────────────── 规则/配置层面 ─────────────────────────
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    #[error("unsupported mapping type: {0}")]
    UnsupportedType(String),

    #[error("condition not satisfied")]
    Skipped,

    // 通用占位
    #[error("internal error: {0}")]
    Internal(String),

    #[error("circular dependency detected among mapping rules")]
    CircularDependency,
}

/// 项目统一 Result 别名
pub type Result<T> = std::result::Result<T, MappingError>;
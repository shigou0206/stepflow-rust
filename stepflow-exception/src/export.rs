use crate::registry::ERROR_REGISTRY;
use serde::Serialize;

/// 可序列化的导出结构
#[derive(Debug, Clone, Serialize)]
pub struct ExportedError {
    pub name: String,
    pub category: String,
    pub description: String,
}

/// 返回所有已注册的错误类型（Vec<ExportedError>）
pub fn all_errors() -> Vec<ExportedError> {
    ERROR_REGISTRY
        .lock()
        .unwrap()
        .values()
        .map(|desc| ExportedError {
            name: desc.name.to_string(),
            category: desc.category.to_string(),
            description: desc.description.to_string(),
        })
        .collect()
}
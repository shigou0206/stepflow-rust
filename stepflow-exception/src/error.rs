/// 错误来源类型（可用于日志/分类）
#[derive(Debug, Clone)]
pub enum ErrorOrigin {
    Tool,
    Engine,
    Mapping,
    Dsl,
    Storage,
    Scheduler,
    Unknown,
}

/// 系统级错误模型
#[derive(Debug, Clone)]
pub struct StepError {
    pub error_type: String,       // 必须可用于 Retry / Catch 匹配
    pub message: String,          // 人类可读的说明
    pub origin: ErrorOrigin,      // 错误源（用于分类分析）
}
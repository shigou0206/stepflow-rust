use crate::register_errors;

/// 注册引擎相关系统级错误类型
pub fn register_engine_errors() {
    register_errors! {
        ChoiceNoMatch => "Engine" => "No matching Choice condition was found",
        EngineAdvanceFailed => "Engine" => "Failed to advance workflow execution",
        UnknownStateType => "Engine" => "Unknown or unsupported state type encountered",
        MissingStateHandler => "Engine" => "No handler registered for this state type"
    };
}
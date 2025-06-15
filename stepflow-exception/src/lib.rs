// 1. 公共错误结构
pub mod error;
pub use error::{StepError, ErrorOrigin};

// 2. 错误注册中心
pub mod registry;
pub use registry::{ErrorDescriptor, get_error_descriptor};

// 3. 注册宏
#[macro_use]
pub mod macros;

// 4. 策略匹配器
pub mod matcher;
pub use matcher::{match_retry, match_catch};

// 5. DSL 校验器
pub mod validate;
pub use validate::{is_valid_error_type, validate_retry_list, validate_catch_list};

// 6. 错误导出器（下一步）
pub mod export;

// 7. 内置错误注册器
pub mod builtin;
pub use builtin::register_all_builtin_errors;
mod interface;
mod memory;
mod hybrid;

pub use self::interface::{Task, MatchService};
pub use self::memory::MemoryMatchService;
pub use self::hybrid::HybridMatchService;

// 重导出一些常用的类型，方便使用方直接从 match_service 模块导入
pub use std::time::Duration;
pub use serde_json::Value;
pub use chrono::NaiveDateTime; 
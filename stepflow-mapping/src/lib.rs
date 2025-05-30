//! crate 公共入口

// —— 先声明各模块 —— //
pub mod error;
pub mod utils;
pub mod model;      //  ← 提到 re-export 之前
pub mod graph;
pub mod resolver;
pub mod engine;

// —— 再做公开 re-export —— //
pub use crate::model::{MappingDSL, MappingRule, MappingResult, PreserveFields};
pub use crate::engine::MappingEngine;
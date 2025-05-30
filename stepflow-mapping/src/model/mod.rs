//! 公共 re-export，外部只引入 `model::*` 即可

pub mod dsl;
pub mod rule;
pub mod schema;
pub mod result;

// 方便顶层 `lib.rs` 直接 `pub use model::*`
pub use dsl::*;
pub use rule::*;
pub use schema::*;
pub use result::*;
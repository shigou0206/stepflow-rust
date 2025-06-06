pub mod core;
pub mod common;
pub mod tools;

// Re-export commonly used types
pub use core::tool::{Tool, ToolMetadata, RetryPolicy, Validation};
pub use core::error::ToolError;
pub use core::registry::ToolRegistry;
pub use common::config::ToolConfig;
pub use common::context::ToolContext;
pub use common::result::{ToolResult, ToolMetadata as ResultMetadata};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

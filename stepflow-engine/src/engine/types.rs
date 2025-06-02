use serde_json::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowMode {
    Inline,
    Deferred,
}

#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub should_continue: bool,
    pub updated_context: Value,
}

// Re-export StateExecutionResult from handler
pub use crate::handler::context::StateExecutionResult; 
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

#[derive(Debug)]
pub struct StateExecutionResult {
    pub value: Value,
}

impl StateExecutionResult {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
} 
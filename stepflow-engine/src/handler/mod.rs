pub mod task;
pub mod wait;
pub mod pass;
pub mod choice;
pub mod succeed;
pub mod fail;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct HandlerOutcome {
    pub should_continue: bool,
    pub updated_context: Value,
}

pub use task::handle_task;
pub use wait::handle_wait;
pub use pass::handle_pass;
pub use choice::handle_choice;
pub use succeed::handle_succeed;
pub use fail::handle_fail;

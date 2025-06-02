pub mod command;
pub mod generator;
pub mod result;

pub use command::Command;
pub use result::CommandResult;
pub use generator::step_once;

impl Command {
    #[inline]
    pub fn kind(&self) -> &'static str {
        match self {
            Command::ExecuteTask { .. } => "Task",
            Command::Wait { .. } => "Wait",
            Command::Pass { .. } => "Pass",
            Command::Choice { .. } => "Choice",
            Command::Succeed { .. } => "Succeed",
            Command::Fail { .. } => "Fail",
        }
    }
}
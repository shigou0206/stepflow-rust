pub mod base;
pub mod task;
pub mod pass;
pub mod wait;
pub mod choice;
pub mod succeed;
pub mod fail;
pub mod parallel;
pub mod map;

use serde::{Deserialize, Serialize};

pub use base::BaseState;
pub use task::TaskState;
pub use pass::PassState;
pub use wait::WaitState;
pub use choice::ChoiceState;
pub use succeed::SucceedState;
pub use fail::FailState;
pub use parallel::ParallelState;
pub use map::MapState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "camelCase")]
pub enum State {
    Task(TaskState),
    Pass(PassState),
    Wait(WaitState),
    Choice(ChoiceState),
    Succeed(SucceedState),
    Fail(FailState),
    Parallel(ParallelState),
    Map(MapState),
}
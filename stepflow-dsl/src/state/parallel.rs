use serde::{Deserialize, Serialize};

use super::base::BaseState;
use crate::branch::Branch;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelState {
    #[serde(flatten)]
    pub base: BaseState,

    pub branches: Vec<Branch>,

    #[serde(default)]
    pub max_concurrency: Option<u32>,
}
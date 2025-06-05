use serde::{Deserialize, Serialize};

use super::base::BaseState;
use crate::branch::Branch;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapState {
    #[serde(flatten)]
    pub base: BaseState,

    pub items_path: String,

    pub iterator: Branch,

    #[serde(default)]
    pub max_concurrency: Option<u32>,
}
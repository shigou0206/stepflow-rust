use serde::{Deserialize, Serialize};

use super::base::BaseState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SucceedState {
    #[serde(flatten)]
    pub base: BaseState,
}
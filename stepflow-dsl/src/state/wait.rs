use serde::{Deserialize, Serialize};

use super::base::BaseState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WaitState {
    #[serde(flatten)]
    pub base: BaseState,

    #[serde(default)]
    pub seconds: Option<u64>,

    #[serde(default)]
    pub timestamp: Option<String>,
}
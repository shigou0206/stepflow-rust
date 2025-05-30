use serde::{Deserialize, Serialize};

use super::base::BaseState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FailState {
    #[serde(flatten)]
    pub base: BaseState,

    #[serde(default)]
    pub error: Option<String>,

    #[serde(default)]
    pub cause: Option<String>,
}
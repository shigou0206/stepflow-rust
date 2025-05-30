use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::base::BaseState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PassState {
    #[serde(flatten)]
    pub base: BaseState,

    #[serde(default)]
    pub result: Option<Value>,

    #[serde(default)]
    pub result_path: Option<String>,
}
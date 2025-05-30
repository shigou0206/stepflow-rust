use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::base::BaseState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TaskState {
    #[serde(flatten)]
    pub base: BaseState,

    pub resource: String,

    #[serde(default)]
    pub parameters: Option<Value>,

    #[serde(default)]
    pub execution_config: Option<Value>,

    #[serde(default)]
    pub heartbeat_seconds: Option<u32>,

    #[serde(default)]
    pub heartbeat_expr: Option<String>,
}

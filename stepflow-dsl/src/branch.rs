use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::state::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Branch {
    pub start_at: String,
    pub states: HashMap<String, State>,
}
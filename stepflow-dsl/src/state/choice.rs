use serde::{Deserialize, Serialize};

use super::base::BaseState;
use crate::logic::ChoiceRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceState {
    #[serde(flatten)]
    pub base: BaseState,

    pub choices: Vec<ChoiceRule>,

    #[serde(default)]
    pub default_next: Option<String>,
}
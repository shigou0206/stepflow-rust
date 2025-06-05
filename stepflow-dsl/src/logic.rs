use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceLogic {
    pub and_: Option<Vec<ChoiceLogic>>,
    pub or_: Option<Vec<ChoiceLogic>>,
    pub not_: Option<Box<ChoiceLogic>>,
    pub variable: Option<String>,
    pub operator: Option<String>,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceRule {
    pub condition: ChoiceLogic,
    pub next: String,
}
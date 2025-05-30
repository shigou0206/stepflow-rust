use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChoiceLogic {
    #[serde(rename = "And", default)]
    pub and_: Option<Vec<ChoiceLogic>>,
    #[serde(rename = "Or", default)]
    pub or_: Option<Vec<ChoiceLogic>>,
    #[serde(rename = "Not", default)]
    pub not_: Option<Box<ChoiceLogic>>,
    #[serde(default)]
    pub variable: Option<String>,
    #[serde(default)]
    pub operator: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChoiceRule {
    pub condition: ChoiceLogic,
    pub next: String,
}
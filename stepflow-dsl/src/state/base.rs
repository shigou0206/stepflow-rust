use serde::{Deserialize, Serialize};
use stepflow_mapping::MappingDSL;

use stepflow_dto::dto::error_policy::{RetryPolicy, CatchPolicy};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseState {
    #[serde(default)]
    pub comment: Option<String>,

    #[serde(default)]
    pub input_mapping: Option<MappingDSL>,

    #[serde(default)]
    pub parameter_mapping: Option<MappingDSL>, 

    #[serde(default)]
    pub output_mapping: Option<MappingDSL>,

    #[serde(default)]
    pub retry: Option<Vec<RetryPolicy>>,

    #[serde(default)]
    pub catch: Option<Vec<CatchPolicy>>,

    #[serde(default)]
    pub next: Option<String>,

    #[serde(default)]
    pub end: Option<bool>,
}

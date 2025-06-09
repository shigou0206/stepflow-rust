use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInputPayload {
    pub parameters: Value,
    pub resource: String,
}

// impl ToolInputPayload {
//     pub fn build(resource: &str, raw: &Value) -> Result<Self, String> {
//         match raw {
//             Value::Object(map) => {
//                 Ok(Self {
//                     parameters: map.get("parameters").cloned().unwrap_or(Value::Null),
//                     resource: resource.to_string(),
//                 })
//             }
//             _ => Err("Expected an object with `parameters`".to_string()),
//         }
//     }
// }
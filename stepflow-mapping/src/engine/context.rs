use serde_json::{Map, Value};
use crate::model::result::MappingStepSnapshot;

pub struct MappingContext {
    pub output: Map<String, Value>,
    pub steps:  Vec<MappingStepSnapshot>,
}

impl MappingContext {
    pub fn new(init: Map<String, Value>) -> Self {
        Self { output: init, steps: vec![] }
    }
}
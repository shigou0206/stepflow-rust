use std::collections::HashMap;
use std::sync::Arc;
use crate::handler::traits::StateHandler;

pub struct StateHandlerRegistry {
    handlers: HashMap<&'static str, Arc<dyn StateHandler>>,
}

impl StateHandlerRegistry {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    pub fn register(mut self, state_type: &'static str, handler: Arc<dyn StateHandler>) -> Self {
        self.handlers.insert(state_type, handler);
        self
    }

    pub fn get(&self, state_type: &str) -> Option<&Arc<dyn StateHandler>> {
        self.handlers.get(state_type)
    }
}
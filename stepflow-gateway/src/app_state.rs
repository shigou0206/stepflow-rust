use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use stepflow_engine::engine::WorkflowEngine;
use stepflow_match::service::MatchService;
use stepflow_storage::db::DynPM;
use stepflow_hook::EngineEventDispatcher;
use stepflow_eventbus::core::bus::EventBus;
#[derive(Clone)]
pub struct AppState {
    pub persist: DynPM,
    pub engines: Arc<Mutex<HashMap<String, WorkflowEngine>>>,
    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub match_service: Arc<dyn MatchService>,
    pub event_bus: Arc<dyn EventBus>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("engines", &"Mutex<...>")
            .field("event_dispatcher", &"EventDispatcher")
            .field("match_service", &"MatchService")
            .finish()
    }
}
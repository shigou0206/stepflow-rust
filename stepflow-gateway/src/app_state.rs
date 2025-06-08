use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use stepflow_engine::engine::WorkflowEngine;
use stepflow_match::service::MatchService;
use stepflow_match::queue::DynPM;
use stepflow_hook::EngineEventDispatcher;

#[derive(Clone)]
pub struct AppState {
    pub persist: DynPM,
    pub engines:   Arc<Mutex<HashMap<String, WorkflowEngine>>>,
    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub match_service: Arc<dyn MatchService>,
}
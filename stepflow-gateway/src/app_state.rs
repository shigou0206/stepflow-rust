use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use stepflow_engine::engine::WorkflowEngine;
use stepflow_match::service::MatchService;   
use stepflow_match::queue::{MemoryQueue, PersistentStore, TaskQueue};
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_hook::EngineEventDispatcher;

#[derive(Clone)]
pub struct AppState {
    pub persist:   Arc<dyn PersistenceManager>,
    pub engines:   Arc<Mutex<HashMap<String, WorkflowEngine<PersistentStore, MemoryQueue>>>>,
    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub match_service: Arc<dyn MatchService>,
    // pub queue: Arc<dyn TaskQueue>,
    // pub memory: Arc<MemoryQueue>,
}
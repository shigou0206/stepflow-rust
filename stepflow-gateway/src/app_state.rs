use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use sqlx::SqlitePool;
use stepflow_engine::engine::{WorkflowEngine, MemoryQueue, PersistentStore};
use stepflow_engine::match_service::MatchService;
use stepflow_storage::PersistenceManager;
use stepflow_hook::EngineEventDispatcher;

#[derive(Clone)]
pub struct AppState {
    pub pool:      SqlitePool,
    pub persist:   Arc<dyn PersistenceManager>,
    pub engines:   Arc<Mutex<HashMap<String, WorkflowEngine<PersistentStore, MemoryQueue>>>>,
    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub match_service: Arc<dyn MatchService>,
}
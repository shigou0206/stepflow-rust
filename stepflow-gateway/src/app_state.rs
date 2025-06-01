use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use sqlx::SqlitePool;
use stepflow_engine::engine::{WorkflowEngine, memory_stub::{MemoryStore, MemoryQueue}};
use stepflow_storage::PersistenceManager;
use stepflow_hook::EngineEventDispatcher;

#[derive(Clone)]
pub struct AppState {
    pub pool:      SqlitePool,
    pub persist:   Arc<dyn PersistenceManager>,
    pub engines:   Arc<Mutex<HashMap<String, WorkflowEngine<MemoryStore, MemoryQueue>>>>,
    pub event_dispatcher: Arc<EngineEventDispatcher>,
}
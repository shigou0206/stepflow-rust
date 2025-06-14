use std::{sync::Arc, str::FromStr, time::Duration, collections::HashMap};
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

use stepflow_gateway::app_state::AppState;
use stepflow_gateway::service::execution::ExecutionSqlxSvc;
use stepflow_match::service::{MemoryMatchService, HybridMatchService, PersistentMatchService};
use stepflow_match::queue::PersistentStore;
use stepflow_hook::EngineEventDispatcher;
use stepflow_hook::impls::log_hook::LogHook;
use stepflow_hook::impls::metrics_hook::MetricsHook;
use stepflow_hook::impls::persist_hook::PersistHook;
use stepflow_sqlite::SqliteStorageManager;
use prometheus::Registry;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use stepflow_eventbus::impls::local::LocalEventBus;
use stepflow_eventbus::core::bus::EventBus;

static EXECUTION_SVC: OnceCell<ExecutionSqlxSvc> = OnceCell::new();
static GLOBAL_EVENT_BUS: OnceCell<Arc<dyn EventBus>> = OnceCell::new();

pub async fn init_app_state(db_path: &str) {
    let db_options = SqliteConnectOptions::from_str(db_path)
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePool::connect_with(db_options).await.unwrap();
    let persist = Arc::new(SqliteStorageManager::new(pool.clone()));

    let event_bus = Arc::new(LocalEventBus::new(100));

    let dispatcher = EngineEventDispatcher::new(vec![
        LogHook::new(),
        MetricsHook::new(&Registry::new()),
        PersistHook::new(persist.clone(), persist.clone(), persist.clone()),
    ], event_bus.clone())
    .enable_batch_processing(100, Duration::from_secs(1));
    let event_dispatcher = Arc::new(dispatcher);

    let memory_match = MemoryMatchService::new();
    let persistent_store = Arc::new(PersistentStore::new(persist.clone()));
    let persistent_match = PersistentMatchService::new(persistent_store.clone(), persist.clone());
    let match_service = HybridMatchService::new(memory_match, persistent_match);



    let state = AppState {
        persist,
        engines: Arc::new(Mutex::new(HashMap::new())),
        event_dispatcher,
        match_service,
        event_bus,
    };

    let svc = ExecutionSqlxSvc::new(Arc::new(state));
    EXECUTION_SVC.set(svc).unwrap();
}

pub fn get_execution_svc() -> &'static ExecutionSqlxSvc {
    EXECUTION_SVC.get().expect("ExecutionSqlxSvc not initialized")
}

pub fn init_event_bus(buffer: usize) {
    let bus = Arc::new(LocalEventBus::new(buffer));
    GLOBAL_EVENT_BUS.set(bus).expect("EventBus already initialized");
}
pub fn get_event_bus() -> Arc<dyn EventBus> {
    GLOBAL_EVENT_BUS
        .get()
        .expect("EventBus not initialized")
        .clone()
}
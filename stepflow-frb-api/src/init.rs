use once_cell::sync::OnceCell;
use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use prometheus::Registry;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use stepflow_eventbus::core::bus::EventBus;
use stepflow_eventbus::impls::local::LocalEventBus;
use stepflow_core::app_state::AppState;
use stepflow_gateway::service::execution::ExecutionSqlxSvc;
use stepflow_hook::EngineEventDispatcher;
use stepflow_hook::impls::log_hook::LogHook;
use stepflow_hook::impls::metrics_hook::MetricsHook;
use stepflow_hook::impls::persist_hook::PersistHook;
use stepflow_match::queue::PersistentStore;
use stepflow_match::service::{HybridMatchService, MemoryMatchService, PersistentMatchService};
use stepflow_sqlite::SqliteStorageManager;
use stepflow_engine::handler::{
    choice::ChoiceHandler, fail::FailHandler, pass::PassHandler, registry::StateHandlerRegistry,
    succeed::SucceedHandler, task::TaskHandler, wait::WaitHandler,
};

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

    let dispatcher = EngineEventDispatcher::new(
        vec![
            LogHook::new(),
            MetricsHook::new(&Registry::new()),
            PersistHook::new(persist.clone(), persist.clone(), persist.clone()),
        ],
        event_bus.clone(),
    )
    .enable_batch_processing(100, Duration::from_secs(1));
    let event_dispatcher = Arc::new(dispatcher);

    let memory_match = MemoryMatchService::new();
    let persistent_store = Arc::new(PersistentStore::new(persist.clone()));
    let persistent_match = PersistentMatchService::new(persistent_store.clone(), persist.clone());
    let match_service = HybridMatchService::new(memory_match, persistent_match);

    let state_handler_registry = StateHandlerRegistry::new()
        .register("task", Arc::new(TaskHandler::new(match_service.clone())))
        .register("wait", Arc::new(WaitHandler::new()))
        .register("pass", Arc::new(PassHandler::new()))
        .register("choice", Arc::new(ChoiceHandler::new()))
        .register("succeed", Arc::new(SucceedHandler::new()))
        .register("fail", Arc::new(FailHandler::new()));
    let state_handler_registry = Arc::new(state_handler_registry);

    let state = AppState {
        persist,
        engines: Arc::new(Mutex::new(HashMap::new())),
        event_dispatcher,
        match_service,
        event_bus,
        state_handler_registry,
    };

    let svc = ExecutionSqlxSvc::new(Arc::new(state));
    EXECUTION_SVC.set(svc).unwrap();
}

pub fn get_execution_svc() -> &'static ExecutionSqlxSvc {
    EXECUTION_SVC
        .get()
        .expect("ExecutionSqlxSvc not initialized")
}

pub fn init_event_bus(buffer: usize) {
    let bus = Arc::new(LocalEventBus::new(buffer));
    GLOBAL_EVENT_BUS
        .set(bus)
        .expect("EventBus already initialized");
}
pub fn get_event_bus() -> Arc<dyn EventBus> {
    GLOBAL_EVENT_BUS
        .get()
        .expect("EventBus not initialized")
        .clone()
}

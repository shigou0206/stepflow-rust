use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode}, SqlitePool};
use stepflow_eventbus::impls::local::LocalEventBus;
use stepflow_hook::{
    EngineEventDispatcher,
    impls::{log_hook::LogHook, metrics_hook::MetricsHook, persist_hook::PersistHook},
};
use stepflow_match::queue::PersistentStore;
use stepflow_match::service::{
    HybridMatchService, MemoryMatchService, PersistentMatchService,
    MatchService, HybridMatchServiceWithEvent
};
use stepflow_sqlite::SqliteStorageManager;
use stepflow_storage::traits::{EventStorage, StateStorage, WorkflowStorage};
use stepflow_engine::handler::{
    choice::ChoiceHandler, fail::FailHandler, pass::PassHandler, registry::StateHandlerRegistry,
    succeed::SucceedHandler, task::TaskHandler, wait::WaitHandler,
};

use stepflow_common::config::{StepflowConfig, StepflowMode};
use prometheus::Registry;

use crate::app_state::AppState;

pub async fn build_app_state(cfg: &StepflowConfig) -> Result<AppState> {
    // ---- DB & Storage ----
    let db_url = format!("sqlite://{}", cfg.db_path);
    let db_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePool::connect_with(db_options).await?;
    let persist = Arc::new(SqliteStorageManager::new(pool));

    // ---- Storage Traits ----
    let workflow_store: Arc<dyn WorkflowStorage> = persist.clone();
    let state_store: Arc<dyn StateStorage> = persist.clone();
    let event_store: Arc<dyn EventStorage> = persist.clone();

    // ---- EventBus ----
    let event_bus = Arc::new(LocalEventBus::new(100));

    // ---- Hook Dispatcher ----
    let mut dispatcher = EngineEventDispatcher::new(
        vec![
            LogHook::new(),
            MetricsHook::new(&Registry::new()),
            PersistHook::new(workflow_store, state_store, event_store),
        ],
        event_bus.clone(),
    );
    dispatcher = dispatcher.enable_batch_processing(100, Duration::from_secs(1));
    let event_dispatcher = Arc::new(dispatcher);

    // ---- Match Service ----
    let match_service: Arc<dyn MatchService> = match cfg.mode {
        StepflowMode::Polling => {
            HybridMatchService::new(
                MemoryMatchService::new(),
                PersistentMatchService::new(
                    Arc::new(PersistentStore::new(persist.clone())),
                    persist.clone(),
                ),
            )
        }
        StepflowMode::EventDriven => {
            let persistent = PersistentMatchService::new(
                Arc::new(PersistentStore::new(persist.clone())),
                persist.clone(),
            );
            HybridMatchServiceWithEvent::new(persistent, event_bus.clone())
        }
    };

    // ---- Handlers ----
    let state_handler_registry = Arc::new(
        StateHandlerRegistry::new()
            .register("task", Arc::new(TaskHandler::new(match_service.clone())))
            .register("wait", Arc::new(WaitHandler::new()))
            .register("pass", Arc::new(PassHandler::new()))
            .register("choice", Arc::new(ChoiceHandler::new()))
            .register("succeed", Arc::new(SucceedHandler::new()))
            .register("fail", Arc::new(FailHandler::new()))
    );

    Ok(AppState {
        persist,
        engines: Default::default(),
        event_dispatcher,
        match_service,
        event_bus,
        state_handler_registry,
    })
}
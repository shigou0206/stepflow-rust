use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::Result;
use std::path::Path;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use stepflow_engine::handler::{
    choice::ChoiceHandler, 
    fail::FailHandler, 
    map::MapHandler, 
    parallel::ParallelHandler, 
    pass::PassHandler, 
    succeed::SucceedHandler, 
    task::TaskHandler, 
    wait::WaitHandler,
    registry::StateHandlerRegistry,
};
use stepflow_eventbus::impls::local::LocalEventBus;
use stepflow_hook::{
    EngineEventDispatcher,
    impls::{log_hook::LogHook, metrics_hook::MetricsHook, persist_hook::PersistHook},
};
use stepflow_match::queue::PersistentStore;
use stepflow_match::service::{
    HybridMatchService, 
    HybridMatchServiceWithEvent, 
    MatchService, 
    MemoryMatchService,
    PersistentMatchService, 
    SubflowMatchService, 
    EventDrivenSubflowMatchService,
};
use stepflow_sqlite::SqliteStorageManager;
use stepflow_storage::traits::{EventStorage, StateStorage, WorkflowStorage};

use crate::service_register::ServiceRegistry;
use crate::service::{
    TemplateSvc, 
    ExecutionSvc, 
    ActivityTaskSvc, 
    WorkflowEventSvc, 
    QueueTaskSvc, 
    TimerSvc, 
    LocalEngineSvc};
use crate::poller_manager::PollerManager;
use prometheus::Registry;
use stepflow_common::config::{StepflowConfig, StepflowExecMode};

use crate::app_state::AppState;


pub fn build_service_registry(state: Arc<AppState>) -> ServiceRegistry {
    ServiceRegistry {
        template: Arc::new(TemplateSvc::new(state.persist.clone())),
        execution: Arc::new(ExecutionSvc::new(state.clone())),
        activity_task: Arc::new(ActivityTaskSvc::new(state.persist.clone())),
        workflow_event: Arc::new(WorkflowEventSvc::new(state.persist.clone())),
        queue_task: Arc::new(QueueTaskSvc::new(state.clone())),
        timer: Arc::new(TimerSvc::new(state.clone())),
        engine: Arc::new(LocalEngineSvc::new(state.clone())),
    }
}

pub async fn build_app_state(cfg: &StepflowConfig) -> Result<AppState> {
    // ---- DB & Storage ----
    println!("🔧 使用数据库路径: {}", cfg.db_path);

    let db_path = Path::new(&cfg.db_path);
    if !db_path.exists() {
        println!("📂 数据库文件不存在，将尝试创建: {}", db_path.display());
    }

    let db_url = format!("sqlite://{}", cfg.db_path);
    let db_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePool::connect_with(db_options).await?;
    println!("✅ SQLite 连接建立成功");

    // 初始化 StorageManager（async）
    let manager = SqliteStorageManager::new(pool).await?;
    println!("✅ StorageManager 初始化完成");

    // ✅ 包成 Arc 并赋值给各个 trait
    let persist = Arc::new(manager);
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
    let match_service: Arc<dyn MatchService> = match cfg.exec_mode {
        StepflowExecMode::Polling => HybridMatchService::new(
            MemoryMatchService::new(),
            PersistentMatchService::new(
                Arc::new(PersistentStore::new(persist.clone())),
                persist.clone(),
            ),
        ),
        StepflowExecMode::EventDriven => {
            let persistent = PersistentMatchService::new(
                Arc::new(PersistentStore::new(persist.clone())),
                persist.clone(),
            );
            HybridMatchServiceWithEvent::new(persistent, event_bus.clone())
        }
    };

    let subflow_match_service: Arc<dyn SubflowMatchService> = match cfg.exec_mode {
        StepflowExecMode::Polling => EventDrivenSubflowMatchService::new(event_bus.clone()),
        StepflowExecMode::EventDriven => EventDrivenSubflowMatchService::new(event_bus.clone()),
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
            .register("map", Arc::new(MapHandler::new(subflow_match_service.clone())))
            .register("parallel", Arc::new(ParallelHandler::new(subflow_match_service.clone()))),
    );

    let mut state = AppState {
        persist: persist.clone(),
        engines: Default::default(),
        event_dispatcher: event_dispatcher.clone(),
        match_service: match_service.clone(),
        event_bus: event_bus.clone(),
        state_handler_registry: state_handler_registry.clone(),
        services: Arc::new(ServiceRegistry::empty()),
    };

    // ④ 构造服务注册表并注入
    let registry = Arc::new(build_service_registry(Arc::new(state.clone())));
    state.services = registry;

    let pollers = PollerManager::new(Arc::new(state.clone()));
    pollers.start_all();

    Ok(state)
}

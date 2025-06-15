use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use tokio::sync::broadcast::Receiver;
use stepflow_engine::engine::WorkflowEngine;
use stepflow_match::service::MatchService;
use stepflow_storage::db::DynPM;
use stepflow_hook::EngineEventDispatcher;
use stepflow_eventbus::core::bus::EventBus;
use stepflow_dto::dto::event_envelope::EventEnvelope;
use stepflow_engine::handler::registry::StateHandlerRegistry;

#[derive(Clone)]
pub struct AppState {
    pub persist: DynPM,
    pub engines: Arc<Mutex<HashMap<String, WorkflowEngine>>>,
    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub match_service: Arc<dyn MatchService>,
    pub event_bus: Arc<dyn EventBus>,
    pub state_handler_registry: Arc<StateHandlerRegistry>,
}

impl AppState {
    pub fn new(
        persist: DynPM,
        event_dispatcher: Arc<EngineEventDispatcher>,
        match_service: Arc<dyn MatchService>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            persist,
            engines: Arc::new(Mutex::new(HashMap::new())),
            event_dispatcher,
            match_service,
            event_bus,
            state_handler_registry: Arc::new(StateHandlerRegistry::new()),
        }
    }

    /// 给外部或内部组件获取一个新的订阅者，
    /// 用来接收所有发往 EventBus 的 EventEnvelope。
    pub fn subscribe_events(&self) -> Receiver<EventEnvelope> {
        self.event_bus.subscribe()
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("engines", &"Mutex<HashMap<...>>")
            .field("event_dispatcher", &"EventDispatcher")
            .field("match_service", &"MatchService")
            .field("event_bus", &"EventBus")
            .field("state_handler_registry", &"StateHandlerRegistry")
            .finish()
    }
}
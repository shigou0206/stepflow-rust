use std::sync::Arc;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_eventbus::core::bus::EventBus;
use stepflow_eventbus::global::GLOBAL_EVENT_BUS;

/// 全局事件广播
pub async fn dispatch_event(event: EngineEvent) {
    if let Some(bus) = GLOBAL_EVENT_BUS.get() {
        if let Err(e) = bus.publish_engine_event(event).await {
            tracing::error!("❌ Failed to publish EngineEvent: {e}");
        }
    } else {
        tracing::warn!("⚠️ GLOBAL_EVENT_BUS not initialized");
    }
}
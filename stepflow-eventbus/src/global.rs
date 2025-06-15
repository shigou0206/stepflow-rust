use std::sync::Arc;
use once_cell::sync::OnceCell;
use stepflow_dto::dto::engine_event::EngineEvent;
use crate::core::bus::EventBus;
use tracing::{error, warn};
use anyhow::{anyhow, Result};

/// 全局事件总线（只允许设置一次）
pub static GLOBAL_EVENT_BUS: OnceCell<Arc<dyn EventBus>> = OnceCell::new();

/// 设置全局事件总线（建议在系统初始化阶段设置一次）
pub fn set_global_event_bus(bus: Arc<dyn EventBus>) -> Result<()> {
    GLOBAL_EVENT_BUS
        .set(bus)
        .map_err(|_| anyhow!("GLOBAL_EVENT_BUS already set"))
}

/// 从全局中获取事件总线（供其他模块使用）
pub fn get_global_event_bus() -> Option<&'static Arc<dyn EventBus>> {
    GLOBAL_EVENT_BUS.get()
}

/// 派发事件（一般供 Task 完成等模块调用）
pub async fn dispatch_event(event: EngineEvent) {
    if let Some(bus) = get_global_event_bus() {
        if let Err(e) = bus.publish_engine_event(event).await {
            error!("❌ Failed to dispatch EngineEvent: {e}");
        }
    } else {
        warn!("⚠️ GLOBAL_EVENT_BUS not initialized");
    }
}
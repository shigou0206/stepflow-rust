use crate::{EngineEvent, EngineEventHandler, ui_event::UiEvent};
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;
use std::sync::Arc;

pub struct WsHook {
    sender: UnboundedSender<UiEvent>,
}

impl WsHook {
    pub fn new(sender: UnboundedSender<UiEvent>) -> Arc<Self> {
        Arc::new(Self { sender })
    }
}

#[async_trait::async_trait]
impl EngineEventHandler for WsHook {
    async fn handle_event(&self, event: EngineEvent) {
        let ui_event = match event {
            EngineEvent::NodeEnter { run_id, state_name, input } => Some(UiEvent {
                event_type: "node_enter".into(),
                run_id,
                state_name: Some(state_name),
                payload: input,
            }),

            EngineEvent::NodeSuccess { run_id, state_name, output } => Some(UiEvent {
                event_type: "node_success".into(),
                run_id,
                state_name: Some(state_name),
                payload: output,
            }),

            EngineEvent::NodeFailed { run_id, state_name, error } => Some(UiEvent {
                event_type: "node_failed".into(),
                run_id,
                state_name: Some(state_name),
                payload: json!({ "error": error }),
            }),

            EngineEvent::WorkflowFinished { run_id, result } => Some(UiEvent {
                event_type: "workflow_finished".into(),
                run_id,
                state_name: None,
                payload: result,
            }),

            _ => None,
        };

        if let Some(evt) = ui_event {
            let _ = self.sender.send(evt); // 忽略发送失败（如接收端关闭）
        }
    }
}
//! signal/manager.rs - 信号管理器：每个 run_id 对应一个独立的 tokio::mpsc 队列

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use stepflow_dto::dto::signal::ExecutionSignal;

pub type SignalSender = mpsc::UnboundedSender<ExecutionSignal>;
pub type SignalReceiver = mpsc::UnboundedReceiver<ExecutionSignal>;

#[derive(Default, Clone)]
pub struct SignalManager {
    inner: Arc<Mutex<HashMap<String, SignalSender>>>,
}

impl SignalManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, run_id: &str) -> SignalReceiver {
        let (tx, rx) = mpsc::unbounded_channel();
        self.inner.lock().await.insert(run_id.to_string(), tx);
        rx
    }

    pub async fn remove(&self, run_id: &str) {
        self.inner.lock().await.remove(run_id);
    }

    pub async fn send(&self, run_id: &str, signal: ExecutionSignal) -> Result<(), String> {
        let map = self.inner.lock().await;
        if let Some(sender) = map.get(run_id) {
            sender.send(signal).map_err(|e| e.to_string())
        } else {
            Err(format!("No signal channel found for run_id: {}", run_id))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_flow() {
        let mgr = SignalManager::new();
        let mut rx = mgr.subscribe("test").await;

        let signal = ExecutionSignal::TimerFired {
            run_id: "test".into(),
            state_name: "Wait1".into(),
        };

        mgr.send("test", signal.clone()).await.unwrap();

        let recv = rx.recv().await.unwrap();
        assert_eq!(format!("{:?}", recv), format!("{:?}", signal));
    }
}
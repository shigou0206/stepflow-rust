use std::sync::Arc;

use crate::app_state::AppState;
use crate::watcher::{
    subflow::SubflowCompletionWatcher,
    timer::TimerPoller,
};

#[derive(Clone)]
pub struct PollerManager {
    subflow_completion: SubflowCompletionWatcher,
    timer: TimerPoller,
}

impl PollerManager {
    pub fn new(app: Arc<AppState>) -> Self {
        Self {
            subflow_completion: SubflowCompletionWatcher::new(app.clone()),
            timer: TimerPoller::new(app.clone()),
        }
    }

    pub fn start_all(&self) {
        let subflow = self.subflow_completion.clone();
        let timer = self.timer.clone();

        // tokio::spawn(async move { subflow.run().await });
        tokio::spawn(async move { timer.run().await });
    }
}
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error};

use crate::app_state::AppState;

/// ✅ 子流程完成检测器：从 AppState 中获取持久化层和引擎服务
#[derive(Clone)]
pub struct SubflowCompletionWatcher {
    app: Arc<AppState>,
}

impl SubflowCompletionWatcher {
    pub fn new(app: Arc<AppState>) -> Self {
        Self { app }
    }

    pub async fn run(&self) {
        loop {
            // 读取已完成的子流程组（全部子流程达到终态）
            match self.app.persist.find_fully_completed_subflow_groups().await {
                Ok(groups) => {
                    for (parent_run_id, state_name) in groups {
                        debug!(
                            %parent_run_id,
                            %state_name,
                            "📦 All subflows in group completed, sending signal"
                        );

                        if let Err(e) = self
                            .app
                            .services
                            .engine
                            .send_subflow_finished(&parent_run_id, &state_name)
                            .await
                        {
                            error!("❌ Failed to signal subflow completion: {e}");
                        }
                    }
                }
                Err(e) => error!("❌ DB query in SubflowCompletionWatcher failed: {e}"),
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
}
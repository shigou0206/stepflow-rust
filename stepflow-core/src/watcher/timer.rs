use std::sync::Arc;
use crate::app_state::AppState;
use tracing::{debug, error};
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct TimerPoller {
    app: Arc<AppState>,
}

impl TimerPoller {
    pub fn new(app: Arc<AppState>) -> Self {
        Self { app }
    }

    pub async fn run(&self) {
        loop {
            // 查询并触发所有 due 的定时器（fire + 更新状态）
            match self.app.services.timer.find_timers_before(chrono::Utc::now(), 100).await {
                Ok(timers) => {
                    for timer in timers {
                        debug!(id = %timer.timer_id, "⏰ Timer ready to fire");

                        // 通知引擎处理该定时器（需要你在 engine service 中实现 handle_timer_fired，如果没有也可改为 send_signal）
                        if let Err(e) = self
                            .app
                            .services
                            .engine
                            .handle_timer_fired(&timer)
                            .await
                        {
                            error!("❌ Failed to handle fired timer: {e}");
                        }
                    }
                }
                Err(e) => error!("❌ Timer poll query failed: {e}"),
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
}
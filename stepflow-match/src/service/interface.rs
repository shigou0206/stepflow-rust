//! service/interface.rs  —— 最终版 MatchService Trait

use async_trait::async_trait;
use serde_json::Value;
use std::{any::Any, sync::Arc, time::Duration};

use stepflow_dto::dto::{match_stats::MatchStats, queue_task::QueueTaskDto};
use stepflow_storage::{
    db::DbBackend,                        // 👈 统一后端
    persistence_manager::PersistenceManager,
};

/// 方便书写的别名
pub type DynPM = Arc<dyn PersistenceManager<DB = DbBackend> + Send + Sync>;

#[async_trait]
pub trait MatchService: Send + Sync {
    /// 允许向下转型
    fn as_any(&self) -> &dyn Any;

    /// 每个队列的实时统计（可选实现，默认空）
    async fn queue_stats(&self) -> Vec<MatchStats> { Vec::new() }

    /// worker 取任务
    async fn poll_task(
        &self,
        queue: &str,
        worker_id: &str,
        timeout: Duration,
    ) -> Option<QueueTaskDto>;

    /// push 任务到队列
    async fn enqueue_task(&self, queue: &str, task: QueueTaskDto) -> Result<(), String>;

    /// 等待任务完成
    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        pm: &DynPM,                // ⚠️ 接口改为 &DynPM
    ) -> Result<Value, String>;
}
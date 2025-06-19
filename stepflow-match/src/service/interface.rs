//! service/interface.rs

use async_trait::async_trait;
use serde_json::Value;
use std::{any::Any, sync::Arc, time::Duration};

use stepflow_dto::dto::{
    match_stats::MatchStats,
    queue_task::{QueueTaskDto, UpdateQueueTaskDto},   // ← 就这俩
};
use stepflow_storage::{db::DbBackend, persistence_manager::PersistenceManager};

pub type DynPM = Arc<dyn PersistenceManager<DB = DbBackend> + Send + Sync>;

#[async_trait]
pub trait MatchService: Send + Sync {
    fn as_any(&self) -> &dyn Any;

    async fn queue_stats(&self) -> Vec<MatchStats> { Vec::new() }

    // ---------------- lifecycle ----------------

    /// 入队，返回 task_id
    async fn enqueue_task(&self, queue: &str, task: QueueTaskDto) -> Result<String, String>;

    /// worker 取任务；取到即标记为 processing
    async fn take_task(
        &self,
        queue: &str,
        worker_id: &str,
        timeout: Duration,
    ) -> Option<QueueTaskDto>;

    /// 任务结束（完成 / 失败 / 取消）
    ///
    /// *用 `(run_id, state_name)` 双键定位任务*，同时带上 **局部 patch**。
    async fn finish_task(
        &self,
        run_id:     &str,
        state_name: &str,
        patch:      UpdateQueueTaskDto,
    ) -> Result<(), String>;

    // ---------------- Engine 专用 ----------------

    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        pm: &DynPM,
    ) -> Result<Value, String>;
}

#[async_trait]
pub trait SubflowMatchService: Send + Sync {
    fn as_any(&self) -> &dyn Any;

    /// 通知子流程已准备好（由 MapHandler / ParallelHandler 调用）
    async fn notify_subflow_ready(
        &self,
        run_id: String,
        parent_run_id: String,
        state_name: String,
    ) -> Result<(), String>;
}
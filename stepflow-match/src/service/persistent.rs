//! persistent_match_service.rs
//! 把基于数据库的 PersistentStore 适配成 MatchService

use std::{any::Any, sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;

use crate::{
    queue::{PersistentStore, TaskStore, DynPM},
    service::MatchService,
};


use stepflow_dto::dto::{
    match_stats::MatchStats,
    queue_task::QueueTaskDto,
};
use stepflow_storage::entities::queue_task::UpdateStoredQueueTask;

/// 真正的服务对象
pub struct PersistentMatchService {
    store:       Arc<PersistentStore>,
    persistence: DynPM,                     // <── 统一后端
}

impl PersistentMatchService {
    pub fn new(store: Arc<PersistentStore>, persistence: DynPM) -> Arc<Self> {
        Arc::new(Self { store, persistence })
    }
}

// ────────────────────────── MatchService 实现 ──────────────────────────
#[async_trait]
impl MatchService for PersistentMatchService {
    fn as_any(&self) -> &dyn Any { self }

    // 队列统计
    async fn queue_stats(&self) -> Vec<MatchStats> {
        vec![MatchStats {
            queue:          "default_task_queue".into(),
            pending_tasks:  0,
            waiting_workers: 0,
        }]
    }

    // 取出一条任务
    async fn poll_task(
        &self,
        _queue: &str,
        _worker_id: &str,
        _timeout: Duration,
    ) -> Option<QueueTaskDto> {
        // 这里只示例读取 1 条 pending 任务
        let tasks = self.persistence
            .find_queue_tasks_by_status("pending", 1, 0)
            .await
            .ok()?;

        let task = tasks.into_iter().next()?;

        // 将状态标记为 processing（无视错误）
        let _ = self.persistence.update_queue_task(
            &task.task_id,
            &UpdateStoredQueueTask {
                status:         Some("processing".into()),
                processing_at:  Some(Some(Utc::now().naive_utc())),
                ..Default::default()
            },
        ).await;

        Some(QueueTaskDto {
            task_id:         task.task_id.into(),
            run_id:          task.run_id,
            state_name:      task.state_name,
            resource:        task.resource,
            task_payload:    task.task_payload,
            status:          task.status,
            attempts:        task.attempts,
            max_attempts:    task.max_attempts,
            priority:        task.priority.map(|p| p as u8),
            timeout_seconds: task.timeout_seconds,
            error_message:   task.error_message,
            last_error_at:   task.last_error_at.map(|dt| dt.and_utc()),
            next_retry_at:   task.next_retry_at.map(|dt| dt.and_utc()),
            queued_at:       task.queued_at.and_utc(),
            processing_at:   task.processing_at.map(|dt| dt.and_utc()),
            completed_at:    task.completed_at.map(|dt| dt.and_utc()),
            failed_at:       task.failed_at.map(|dt| dt.and_utc()),
        })
    }

    // 入队
    async fn enqueue_task(&self, _queue: &str, task: QueueTaskDto) -> Result<(), String> {
        self.store.insert_task(
            &self.persistence,
            &task.run_id,
            &task.state_name,
            &task.resource,
            &task.task_payload.clone().unwrap_or(Value::Null),
        ).await
    }

    // 等待完成（这里直接回显）
    async fn wait_for_completion(
        &self,
        _run_id: &str,
        _state_name: &str,
        input: &Value,
        _pm: &DynPM,             
    ) -> Result<Value, String> {
        Ok(input.clone())
    }
}
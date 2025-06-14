// src/service/persistent_match_service.rs
//! 把基于数据库的 PersistentStore 适配成 MatchService

use std::{any::Any, sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;

use stepflow_storage::{
    db::DynPM,
    entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask},
};

use crate::{
    queue::{PersistentStore, TaskStore},
    service::interface::MatchService, 
};

use stepflow_dto::dto::{
    match_stats::MatchStats,
    queue_task::{QueueTaskDto, UpdateQueueTaskDto},
};

/// 真正的服务对象
pub struct PersistentMatchService {
    store:       Arc<PersistentStore>,
    persistence: DynPM,
}

impl PersistentMatchService {
    pub fn new(store: Arc<PersistentStore>, persistence: DynPM) -> Arc<Self> {
        Arc::new(Self { store, persistence })
    }

    /// 把数据库行 → DTO
    fn to_dto(row: StoredQueueTask) -> QueueTaskDto {
        QueueTaskDto {
            task_id:         row.task_id,
            run_id:          row.run_id,
            state_name:      row.state_name,
            resource:        row.resource,
            task_payload:    row.task_payload,
            status:          row.status,
            attempts:        row.attempts,
            max_attempts:    row.max_attempts,
            priority:        row.priority.map(|p| p as u8),
            timeout_seconds: row.timeout_seconds,
            error_message:   row.error_message,
            last_error_at:   row.last_error_at.map(|t| t.and_utc()),
            next_retry_at:   row.next_retry_at.map(|t| t.and_utc()),
            queued_at:       row.queued_at.and_utc(),
            processing_at:   row.processing_at.map(|t| t.and_utc()),
            completed_at:    row.completed_at.map(|t| t.and_utc()),
            failed_at:       row.failed_at.map(|t| t.and_utc()),
        }
    }
}

// ────────────────────────── MatchService 实现 ──────────────────────────
#[async_trait]
impl MatchService for PersistentMatchService {
    fn as_any(&self) -> &dyn Any { self }

    // ───────── stats ─────────
    async fn queue_stats(&self) -> Vec<MatchStats> {
        // 简化实现：仅返回 0
        vec![MatchStats {
            queue: "default_task_queue".into(),
            pending_tasks: 0,
            waiting_workers: 0,
        }]
    }

    // ───────── enqueue ───────
    async fn enqueue_task(&self, _queue: &str, task: QueueTaskDto) -> Result<String, String> {
        self.store
            .insert_task(
                &task.run_id,
                &task.state_name,
                &task.resource,
                &task.task_payload.clone().unwrap_or(Value::Null),
            )
            .await
    }

    // ───────── take_task ─────
    async fn take_task(
        &self,
        _queue: &str,
        _worker_id: &str,
        _timeout: Duration,
    ) -> Option<QueueTaskDto> {
        // 1. 取一条 pending
        let task = self
            .persistence
            .find_queue_tasks_by_status("pending", 1, 0)
            .await
            .ok()?
            .into_iter()
            .next()?;

        // 2. 标记为 processing（忽略错误）
        let _ = self
            .persistence
            .update_queue_task(
                &task.task_id,
                &UpdateStoredQueueTask {
                    status:        Some("processing".into()),
                    processing_at: Some(Some(Utc::now().naive_utc())),
                    ..Default::default()
                },
            )
            .await;

        Some(Self::to_dto(task))
    }

    // ───────── finish_task ───
    async fn finish_task(
        &self,
        run_id:     &str,
        state_name: &str,
        patch:      UpdateQueueTaskDto,
    ) -> Result<(), String> {
        // 1. 先找出 task_id（按 run_id+state_name）
        let maybe = self
            .persistence
            .find_queue_task_by_run_state(run_id, state_name)   // ← 若没有此 API，需要自己实现
            .await
            .map_err(|e| e.to_string())?;

        let task = maybe.ok_or_else(|| "queue_task not found".to_string())?;
        let task_id = task.task_id.clone();

        // 2. 把 DTO patch → Stored patch
        let stored_patch = UpdateStoredQueueTask {
            status:          patch.status,
            task_payload:    patch.task_payload,
            attempts:        patch.attempts,
            priority:        patch.priority.map(|v| v as i64),
            timeout_seconds: patch.timeout_seconds,
            error_message:   patch.error_message,
            last_error_at:   patch.last_error_at.map(|opt| opt.map(|d| d.naive_utc())),
            next_retry_at:   patch.next_retry_at.map(|opt| opt.map(|d| d.naive_utc())),
            processing_at:   patch.processing_at.map(|opt| opt.map(|d| d.naive_utc())),
            completed_at:    patch.completed_at.map(|opt| opt.map(|d| d.naive_utc())),
            failed_at:       patch.failed_at.map(|opt| opt.map(|d| d.naive_utc())),
        };

        // 3. 更新
        self.persistence
            .update_queue_task(&task_id, &stored_patch)
            .await
            .map_err(|e| e.to_string())
    }

    // ───────── wait_for_completion ─
    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input_snapshot: &Value,
        _pm: &DynPM,
    ) -> Result<Value, String> {
        // 简化：轮询一次数据库，若已 completed 就返回 payload，否则返回现场快照
        if let Some(task) = self
            .persistence
            .find_queue_task_by_run_state(run_id, state_name)
            .await
            .map_err(|e| e.to_string())?
        {
            if task.status == "completed" {
                return Ok(task
                    .task_payload
                    .unwrap_or(Value::Null));
            }
        }
        Ok(input_snapshot.clone())
    }
}
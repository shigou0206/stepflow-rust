//! 基于数据库的 TaskStore —— 适配 DynPM
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use super::traits::{DynPM, TaskStore};
use stepflow_storage::entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask};

// ── 状态枚举 & 校验 ───────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus { Pending, Processing, Completed, Failed, Retrying }

impl TaskStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending    => "pending",
            Self::Processing => "processing",
            Self::Completed  => "completed",
            Self::Failed     => "failed",
            Self::Retrying   => "retrying",
        }
    }
    fn from_str(s: &str) -> Option<Self> {                // 简要解析
        Some(match s {
            "pending"    => Self::Pending,
            "processing" => Self::Processing,
            "completed"  => Self::Completed,
            "failed"     => Self::Failed,
            "retrying"   => Self::Retrying,
            _ => return None,
        })
    }
    /// 业务层面的状态流转约束
    fn ok_transition(self, to: Self) -> bool {
        use TaskStatus::*;
        matches!(
            (self, to),
            (Pending, Processing) |
            (Processing, Completed | Failed) |
            (Failed, Retrying)   |
            (Retrying, Processing)
        )
    }
}
// ────────────────────────────────────────────────────────────────

pub struct PersistentStore {
    pm: DynPM,
}
impl PersistentStore {
    pub fn new(pm: DynPM) -> Self { Self { pm } }

    /// 在 DB 里找到一条任务
    async fn get_one(&self, run_id: &str, state_name: &str) -> Result<StoredQueueTask, String> {
        let tasks = self.pm
            .find_queue_tasks_by_status("pending", 100, 0)
            .await
            .map_err(|e| e.to_string())?;

        tasks.into_iter()
             .find(|t| t.run_id == run_id && t.state_name == state_name)
             .ok_or_else(|| "task not found".into())
    }
}

#[async_trait]
impl TaskStore for PersistentStore {
    // ---------------- insert ----------------
    async fn insert_task(
        &self,
        _pm: &DynPM,                      // 外部仍传，但内部用 self.pm
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<(), String> {
        let now = Utc::now().naive_utc();
        let task = StoredQueueTask {
            task_id: Uuid::new_v4().to_string(),
            run_id: run_id.into(),
            state_name: state_name.into(),
            resource: resource.into(),
            task_payload: Some(input.clone()),
            status: TaskStatus::Pending.as_str().into(),
            attempts: 0,
            max_attempts: 3,
            priority: Some(0),
            timeout_seconds: Some(300),
            error_message: None,
            last_error_at: None,
            next_retry_at: None,
            queued_at: now,
            processing_at: None,
            completed_at: None,
            failed_at: None,
            created_at: now,
            updated_at: now,
        };
        self.pm.create_queue_task(&task).await.map_err(|e| e.to_string())
    }

    // ---------------- update ----------------
    async fn update_task_status(
        &self,
        _pm: &DynPM,
        run_id: &str,
        state_name: &str,
        new_status_str: &str,
        result: &Value,
    ) -> Result<(), String> {
        let task = self.get_one(run_id, state_name).await?;
        let now = Utc::now().naive_utc();

        // 转换 + 校验
        let new_status = TaskStatus::from_str(new_status_str)
            .ok_or_else(|| format!("invalid status {new_status_str}"))?;
        let cur_status = TaskStatus::from_str(&task.status)
            .ok_or("invalid current status")?;
        if !cur_status.ok_transition(new_status) {
            return Err(format!("invalid transition {cur_status:?} -> {new_status:?}"));
        }

        // 组装更新
        let mut ch = UpdateStoredQueueTask {
            status: Some(new_status.as_str().into()),
            task_payload: Some(Some(result.clone())),
            ..Default::default()
        };

        match new_status {
            TaskStatus::Processing => ch.processing_at = Some(Some(now)),
            TaskStatus::Completed  => ch.completed_at  = Some(Some(now)),
            TaskStatus::Failed => {
                ch.failed_at = Some(Some(now));
                ch.error_message = Some(Some(result.to_string()));
                ch.attempts = Some(task.attempts + 1);
            }
            TaskStatus::Retrying => {
                ch.last_error_at = Some(Some(now));
                ch.next_retry_at = Some(Some(now + chrono::Duration::seconds(30)));
                ch.error_message = Some(Some(result.to_string()));
                ch.attempts = Some(task.attempts + 1);
            }
            _ => {}
        }

        self.pm.update_queue_task(&task.task_id, &ch)
               .await
               .map_err(|e| e.to_string())
    }
}
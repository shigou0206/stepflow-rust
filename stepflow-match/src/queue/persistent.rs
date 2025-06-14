// use async_trait::async_trait;
// use chrono::Utc;
// use serde_json::Value;
// use uuid::Uuid;

// use super::traits::TaskStore;
// use stepflow_storage::entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask};
// use stepflow_storage::db::DynPM;

// // ── 状态枚举 & 校验 ───────────────────────────────────────────────
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum TaskStatus { Pending, Processing, Completed, Failed, Retrying }

// impl TaskStatus {
//     fn as_str(self) -> &'static str {
//         match self {
//             Self::Pending    => "pending",
//             Self::Processing => "processing",
//             Self::Completed  => "completed",
//             Self::Failed     => "failed",
//             Self::Retrying   => "retrying",
//         }
//     }
//     fn from_str(s: &str) -> Option<Self> { 
//         Some(match s {
//             "pending"    => Self::Pending,
//             "processing" => Self::Processing,
//             "completed"  => Self::Completed,
//             "failed"     => Self::Failed,
//             "retrying"   => Self::Retrying,
//             _ => return None,
//         })
//     }
//     /// 业务层面的状态流转约束
//     fn ok_transition(self, to: Self) -> bool {
//         use TaskStatus::*;
//         matches!(
//             (self, to),
//             (Pending, Processing) |
//             (Processing, Completed | Failed) |
//             (Failed, Retrying)   |
//             (Retrying, Processing)
//         )
//     }
// }
// // ────────────────────────────────────────────────────────────────

// pub struct PersistentStore {
//     pm: DynPM,
// }
// impl PersistentStore {
//     pub fn new(pm: DynPM) -> Self { Self { pm } }

//     /// 在 DB 里找到一条任务
//     async fn get_one(&self, run_id: &str, state_name: &str) -> Result<StoredQueueTask, String> {
//         let tasks = self.pm
//             .find_queue_tasks_by_status("pending", 100, 0)
//             .await
//             .map_err(|e| e.to_string())?;

//         tasks.into_iter()
//              .find(|t| t.run_id == run_id && t.state_name == state_name)
//              .ok_or_else(|| "task not found".into())
//     }
// }

// #[async_trait]
// impl TaskStore for PersistentStore {
//     // ---------------- insert ----------------
//     async fn insert_task(
//         &self,
//         run_id: &str,
//         state_name: &str,
//         resource: &str,
//         input: &Value,
//     ) -> Result<String, String> {
//         let now = Utc::now().naive_utc();
//         let task_id = Uuid::new_v4().to_string();
//         let rec = StoredQueueTask {
//             task_id: task_id.clone(),
//             run_id: run_id.into(),
//             state_name: state_name.into(),
//             resource: resource.into(),
//             task_payload: Some(input.clone()),
//             status: "pending".into(),
//             attempts: 0,
//             max_attempts: 3,
//             priority: Some(0),
//             timeout_seconds: Some(300),
//             error_message: None,
//             last_error_at: None,
//             next_retry_at: None,
//             queued_at: now,
//             processing_at: None,
//             completed_at: None,
//             failed_at: None,
//             created_at: now,
//             updated_at: now,
//         };
//         self.pm.create_queue_task(&rec)
//             .await
//             .map_err(|e| e.to_string())?;
//         Ok(task_id)
//     }

//     // ---------------- update ----------------
//     async fn update_task_status(
//         &self,
//         run_id: &str,
//         state_name: &str,
//         new_status_str: &str,
//         result: &Value,
//     ) -> Result<(), String> {
//         let task = self.get_one(run_id, state_name).await?;
//         let now = Utc::now().naive_utc();

//         // 转换 + 校验
//         let new_status = TaskStatus::from_str(new_status_str)
//             .ok_or_else(|| format!("invalid status {new_status_str}"))?;
//         let cur_status = TaskStatus::from_str(&task.status)
//             .ok_or("invalid current status")?;
//         if !cur_status.ok_transition(new_status) {
//             return Err(format!("invalid transition {cur_status:?} -> {new_status:?}"));
//         }

//         // 组装更新
//         let mut ch = UpdateStoredQueueTask {
//             status: Some(new_status.as_str().into()),
//             task_payload: Some(Some(result.clone())),
//             ..Default::default()
//         };

//         match new_status {
//             TaskStatus::Processing => ch.processing_at = Some(Some(now)),
//             TaskStatus::Completed  => ch.completed_at  = Some(Some(now)),
//             TaskStatus::Failed => {
//                 ch.failed_at = Some(Some(now));
//                 ch.error_message = Some(Some(result.to_string()));
//                 ch.attempts = Some(task.attempts + 1);
//             }
//             TaskStatus::Retrying => {
//                 ch.last_error_at = Some(Some(now));
//                 ch.next_retry_at = Some(Some(now + chrono::Duration::seconds(30)));
//                 ch.error_message = Some(Some(result.to_string()));
//                 ch.attempts = Some(task.attempts + 1);
//             }
//             _ => {}
//         }

//         self.pm.update_queue_task(&task.task_id, &ch)
//                .await
//                .map_err(|e| e.to_string())
//     }
// }

//! persistence/persistent_store.rs
//! -------------------------------------------------------------
//! 持久化任务存储（QueueTask）——面向 `MatchService` / `TaskStore`
//! 1. **insert_task**  —— 运行时在 Task 节点挂起时调用
//! 2. **update_task_status** —— worker 回调 /update 时调用
//!    * 只做一次 **原子 UPDATE**，不再先 SELECT
//!    * 依赖 queue_tasks 上 (run_id, state_name) 的唯一索引
//! -------------------------------------------------------------

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::traits::TaskStore;
use stepflow_storage::{
    db::DynPM,
    entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask},
};

/* ====================================================================== */
/*                               Status Enum                             */
/* ====================================================================== */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}

impl TaskStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "pending" => Self::Pending,
            "processing" => Self::Processing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "retrying" => Self::Retrying,
            _ => return None,
        })
    }

    /// 检查业务层面的合法流转
    pub fn ok_transition(self, to: Self) -> bool {
        use TaskStatus::*;
        matches!(
            (self, to),
            (Pending, Processing)
                | (Processing, Completed | Failed)
                | (Failed, Retrying)
                | (Retrying, Processing)
        )
    }

    /// 用于原子更新的“前置状态”推导
    pub fn expected_prev(self) -> Self {
        use TaskStatus::*;
        match self {
            Processing => Pending,
            Completed | Failed => Processing,
            Retrying => Failed,
            Pending => Pending, // 理论上不会用到
        }
    }
}

/* ====================================================================== */
/*                               Store Impl                               */
/* ====================================================================== */
/// SQLite / PostgreSQL 都可复用，只要 `DynPM` 实现了对应接口即可。
pub struct PersistentStore {
    pm: DynPM,
}

impl PersistentStore {
    pub fn new(pm: DynPM) -> Self {
        Self { pm }
    }
}

#[async_trait]
impl TaskStore for PersistentStore {
    /* ---------------------------- insert_task ------------------------- */
    async fn insert_task(
        &self,
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<String, String> {
        let now = Utc::now().naive_utc();
        let task_id = Uuid::new_v4().to_string();

        let rec = StoredQueueTask {
            task_id: task_id.clone(),
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

        self.pm
            .create_queue_task(&rec)
            .await
            .map_err(|e| e.to_string())?;

        Ok(task_id)
    }

    /* ------------------------- update_task_status --------------------- */
    async fn update_task_status(
        &self,
        run_id: &str,
        state_name: &str,
        new_status_str: &str,
        result: &Value,
    ) -> Result<(), String> {
        // -------- 1. 解析 & 校验目标状态 --------
        let new_status = TaskStatus::from_str(new_status_str)
            .ok_or_else(|| format!("invalid status {new_status_str}"))?;

        // 允许的状态机转移由调用方（gateway）保证，这里只用 expected_prev() 做乐观锁。
        let expected_prev = new_status.expected_prev();
        let now = Utc::now().naive_utc();

        // -------- 2. 组装变更字段 --------
        let mut upd = UpdateStoredQueueTask {
            status: Some(new_status.as_str().into()),
            task_payload: Some(Some(result.clone())),
            ..Default::default()
        };

        match new_status {
            TaskStatus::Processing => {
                upd.processing_at = Some(Some(now));
            }
            TaskStatus::Completed => {
                upd.completed_at = Some(Some(now));
            }
            TaskStatus::Failed => {
                upd.failed_at = Some(Some(now));
                upd.error_message = Some(Some(result.to_string()));
                // attempts +1 由 SQL 里 `attempts = attempts + 1` 更佳，这里简单用旧值+1
                upd.attempts = Some(1); // 占位，SQL 内部叠加更干净
            }
            TaskStatus::Retrying => {
                upd.last_error_at = Some(Some(now));
                upd.next_retry_at = Some(Some(now + ChronoDuration::seconds(30)));
                upd.error_message = Some(Some(result.to_string()));
                upd.attempts = Some(1);
            }
            _ => {}
        }

        // -------- 3. 原子 UPDATE --------
        let affected = self
            .pm
            .update_task_by_run_state(
                run_id,
                state_name,
                Some(expected_prev.as_str()),
                &upd,
            )
            .await
            .map_err(|e| e.to_string())?;

        // -------- 4. 幂等校验 --------
        if affected == 0 {
            Err("task not found or status already changed".into())
        } else {
            Ok(())
        }
    }
}
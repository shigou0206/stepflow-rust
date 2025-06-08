//! 基于内存的 TaskQueue 实现
//! ------------------------------------------------------------
//! 依赖：
//!   - super::traits::{TaskQueue, DynPM}
//!   - stepflow_storage::db::DbBackend  (间接出现在 DynPM 里)
//!
//! 重点：所有队列操作都带上 `persistence: &DynPM`，
//!      如果暂时不用可以忽略（下划线前缀）。

use chrono::{DateTime, Utc, Duration as ChronoDuration};
use tokio::sync::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use async_trait::async_trait;

use super::traits::{TaskQueue, DynPM};

/// 内部任务对象；仅存储队列必要信息
#[derive(Debug, Clone)]
pub struct QueueTask {
    pub run_id:     String,
    pub state_name: String,
    pub priority:   u8,
    pub created_at: DateTime<Utc>,
}

impl QueueTask {
    pub fn new(run_id: String, state_name: String) -> Self {
        Self {
            run_id,
            state_name,
            priority: 128,          // 默认优先级
            created_at: Utc::now(),
        }
    }
    pub fn with_priority(run: String, state: String, prio: u8) -> Self {
        Self {
            run_id: run,
            state_name: state,
            priority: prio,
            created_at: Utc::now(),
        }
    }
}

/// 纯内存环形队列
pub struct MemoryQueue {
    capacity: usize,
    tasks:    Mutex<VecDeque<QueueTask>>,
}

impl MemoryQueue {
    pub fn new() -> Self { Self::with_capacity(1_000) }
    pub fn with_capacity(capacity: usize) -> Self {
        Self { capacity, tasks: Mutex::new(VecDeque::with_capacity(capacity)) }
    }

    // ==== 便捷辅助 ====

    pub async fn clear(&self) { self.tasks.lock().await.clear(); }
    pub async fn len(&self) -> usize { self.tasks.lock().await.len() }
    pub async fn is_empty(&self) -> bool { self.tasks.lock().await.is_empty() }

    pub async fn peek(&self) -> Option<QueueTask> {
        self.tasks.lock().await.front().cloned()
    }
    pub async fn contains(&self, run_id: &str) -> bool {
        self.tasks.lock().await.iter().any(|t| t.run_id == run_id)
    }

    // ==== 批量 ====

    pub async fn push_batch(&self, mut tasks: Vec<QueueTask>) -> Result<(), String> {
        let mut q = self.tasks.lock().await;
        if q.len() + tasks.len() > self.capacity {
            return Err("Queue capacity exceeded".into());
        }
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        q.extend(tasks);
        Ok(())
    }

    pub async fn pop_batch(&self, n: usize) -> Vec<QueueTask> {
        let mut q = self.tasks.lock().await;
        (0..n.min(q.len())).filter_map(|_| q.pop_front()).collect()
    }

    /// 清除创建时间超过 `max_age` 的任务，返回删除数量
    pub async fn cleanup_expired(&self, max_age: ChronoDuration) -> usize {
        let now = Utc::now();
        let mut q = self.tasks.lock().await;
        let before = q.len();
        q.retain(|t| now - t.created_at <= max_age);
        before - q.len()
    }

    /// 按优先级插入
    pub async fn push_with_priority(
        &self,
        run_id: &str,
        state_name: &str,
        priority: u8,
    ) -> Result<(), String> {
        let mut q = self.tasks.lock().await;
        if q.len() >= self.capacity {
            return Err("Queue capacity exceeded".into());
        }
        let pos = q.iter().position(|t| t.priority <= priority).unwrap_or(q.len());
        q.insert(pos, QueueTask::with_priority(run_id.into(), state_name.into(), priority));
        Ok(())
    }
}

//// ------------------------------------------------------------------
// TaskQueue 实现 —— 只把 persistence 类型改成 DynPM
//// ------------------------------------------------------------------

#[async_trait]
impl TaskQueue for MemoryQueue {
    async fn push(
        &self,
        _persistence: &DynPM,
        run_id: &str,
        state_name: &str,
    ) -> Result<(), String> {
        let mut q = self.tasks.lock().await;
        if q.len() >= self.capacity {
            return Err("Queue is full".into());
        }
        q.push_back(QueueTask::new(run_id.into(), state_name.into()));
        Ok(())
    }

    async fn pop(
        &self,
        _persistence: &DynPM,
    ) -> Result<Option<(String, String)>, String> {
        let mut q = self.tasks.lock().await;
        Ok(q.pop_front().map(|t| (t.run_id, t.state_name)))
    }
}

//// ------------------------------------------------------------------
// 单元测试（保持不变，只是把 use super::* 留下即可）
//// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use chrono::Duration as ChronoDuration;

    #[test]
    fn memory_queue_basic_ops() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let q = MemoryQueue::with_capacity(10);

            q.push_with_priority("rid1", "state1", 100).await.unwrap();
            q.push_with_priority("rid2", "state2", 200).await.unwrap();
            q.push_with_priority("rid3", "state3", 150).await.unwrap();

            assert_eq!(q.len().await, 3);
            assert_eq!(q.peek().await.unwrap().run_id, "rid2");

            let batch = q.pop_batch(2).await;
            assert_eq!(batch.len(), 2);
            assert_eq!(q.len().await, 1);

            q.clear().await;
            assert!(q.is_empty().await);
        });
    }

    #[test]
    fn memory_queue_cleanup() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let q = MemoryQueue::with_capacity(5);
            for i in 0..5 {
                q.push_with_priority(&format!("rid{i}"), &format!("state{i}"), 100).await.unwrap();
            }
            let removed = q.cleanup_expired(ChronoDuration::seconds(-1)).await;
            assert_eq!(removed, 5);
            assert!(q.is_empty().await);
        });
    }
}
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use tokio::sync::Mutex;
use std::sync::Arc;
use stepflow_storage::persistence_manager::PersistenceManager;
use std::collections::VecDeque;
use async_trait::async_trait;

use super::traits::TaskQueue;

#[derive(Debug, Clone)]
pub struct QueueTask {
    pub run_id: String,
    pub state_name: String,
    pub priority: u8,
    pub created_at: DateTime<Utc>,
}

impl QueueTask {
    fn new(run_id: String, state_name: String) -> Self {
        Self {
            run_id,
            state_name,
            priority: 128, // 默认中等优先级
            created_at: Utc::now(),
        }
    }

    fn with_priority(run_id: String, state_name: String, priority: u8) -> Self {
        Self {
            run_id,
            state_name,
            priority,
            created_at: Utc::now(),
        }
    }
}

pub struct MemoryQueue {
    tasks: Mutex<VecDeque<QueueTask>>,
    capacity: usize,
}

impl MemoryQueue {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tasks: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub async fn clear(&self) {
        self.tasks.lock().await.clear();
    }

    pub async fn len(&self) -> usize {
        self.tasks.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.tasks.lock().await.is_empty()
    }

    // 查询方法
    pub async fn peek(&self) -> Option<QueueTask> {
        self.tasks.lock().await.front().cloned()
    }

    pub async fn contains(&self, run_id: &str) -> bool {
        self.tasks.lock().await
            .iter()
            .any(|task| task.run_id == run_id)
    }

    // 批量操作方法
    pub async fn push_batch(&self, mut tasks: Vec<QueueTask>) -> Result<(), String> {
        let mut queue = self.tasks.lock().await;
        if queue.len() + tasks.len() > self.capacity {
            return Err("Queue capacity exceeded".to_string());
        }

        // 按优先级排序，高优先级在前
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        queue.extend(tasks);
        Ok(())
    }

    pub async fn pop_batch(&self, max_count: usize) -> Vec<QueueTask> {
        let mut queue = self.tasks.lock().await;
        let count = max_count.min(queue.len());
        (0..count).filter_map(|_| queue.pop_front()).collect()
    }

    // 过期清理
    pub async fn cleanup_expired(&self, max_age: ChronoDuration) -> usize {
        let now = Utc::now();
        let mut queue = self.tasks.lock().await;
        let original_len = queue.len();
        queue.retain(|task| now - task.created_at <= max_age);
        original_len - queue.len()
    }

    // 优先级相关
    pub async fn push_with_priority(&self, run_id: &str, state_name: &str, priority: u8) -> Result<(), String> {
        let task = QueueTask::with_priority(run_id.to_owned(), state_name.to_owned(), priority);
        let mut queue = self.tasks.lock().await;
        
        if queue.len() >= self.capacity {
            return Err("Queue capacity exceeded".to_string());
        }

        // 找到合适的插入位置（按优先级降序）
        let pos = queue.iter()
            .position(|t| t.priority <= priority)
            .unwrap_or(queue.len());
        
        queue.insert(pos, task);
        Ok(())
    }
}

#[async_trait]
impl TaskQueue for MemoryQueue {
    async fn push(
        &self,
        _persistence: &Arc<dyn PersistenceManager>,
        run_id: &str,
        state_name: &str,
    ) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        if tasks.len() >= self.capacity {
            return Err("Queue is full".to_string());
        }
        tasks.push_back(QueueTask::new(run_id.to_string(), state_name.to_string()));
        Ok(())
    }

    async fn pop(
        &self,
        _persistence: &Arc<dyn PersistenceManager>,
    ) -> Result<Option<(String, String)>, String> {
        let mut tasks = self.tasks.lock().await;
        Ok(tasks.pop_front().map(|task| (task.run_id, task.state_name)))
    }
} 
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde_json::Value;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::service::{MatchService, Task};
use crate::queue::PersistentStore;
use crate::queue::TaskStore;

/// 将 PersistentStore 包装为 MatchService 的适配器
pub struct PersistentMatchService {
    store: Arc<PersistentStore>,
    persistence: Arc<dyn PersistenceManager>,
}

impl PersistentMatchService {
    pub fn new(store: Arc<PersistentStore>, persistence: Arc<dyn PersistenceManager>) -> Arc<Self> {
        Arc::new(Self { store, persistence })
    }
}

#[async_trait]
impl MatchService for PersistentMatchService {
    async fn poll_task(&self, _queue: &str, _worker_id: &str, _timeout: Duration) -> Option<Task> {
        // 这里可以从数据库中查找 pending 状态的任务
        // 暂简化为 None（需要根据业务模型拓展）
        None
    }

    async fn enqueue_task(&self, _queue: &str, task: Task) -> Result<(), String> {
        // 使用 store.insert_task 创建数据库任务记录
        self.store
            .insert_task(
                &self.persistence,
                &task.run_id,
                &task.state_name,
                &task.task_type,
                &task.input.clone().unwrap_or(Value::Null),
            )
            .await
    }

    async fn wait_for_completion(
        &self,
        _run_id: &str,
        _state_name: &str,
        input: &Value,
        _persistence: Arc<dyn PersistenceManager>,
    ) -> Result<Value, String> {
        // 简单实现：直接返回输入
        Ok(input.clone())
    }
}
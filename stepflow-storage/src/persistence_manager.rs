use async_trait::async_trait;
use crate::transaction::TransactionManager;
use crate::traits::*;

#[async_trait]
pub trait PersistenceManager: 
    WorkflowStorage +
    EventStorage +
    StateStorage +
    ActivityStorage +
    TimerStorage +
    TemplateStorage +
    VisibilityStorage +
    QueueStorage +
    TransactionManager +
    Send + Sync
{}

// 自动实现 PersistenceManager
impl<T> PersistenceManager for T where
    T: WorkflowStorage
        + EventStorage
        + StateStorage
        + ActivityStorage
        + TimerStorage
        + TemplateStorage
        + VisibilityStorage
        + QueueStorage
        + TransactionManager
        + Send
        + Sync
{}
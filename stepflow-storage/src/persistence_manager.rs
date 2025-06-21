// src/persistence_manager.rs
use async_trait::async_trait;
use crate::transaction::TransactionManager;
use crate::traits::*;

#[async_trait]     // 只是为了以后若要加异步方法
pub trait PersistenceManager:
      WorkflowStorage
    + EventStorage
    + StateStorage
    + ActivityStorage
    + TimerStorage
    + TemplateStorage
    + VisibilityStorage
    + QueueStorage
    + TransactionManager           
    + Send + Sync
{}

// 自动 blanket-impl
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
      + Send + Sync {}

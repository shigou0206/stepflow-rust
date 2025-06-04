use async_trait::async_trait;
use crate::error::StorageError;

/// 事务管理器，提供事务相关的操作
#[async_trait]
pub trait TransactionManager {
    /// 开始一个新的事务
    async fn begin_transaction(&self) -> Result<(), StorageError>;

    /// 提交当前事务
    async fn commit(&self) -> Result<(), StorageError>;

    /// 回滚当前事务
    async fn rollback(&self) -> Result<(), StorageError>;
} 
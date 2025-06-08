// src/transaction.rs
use async_trait::async_trait;
use sqlx::{Database, Transaction};
use crate::error::StorageError;

#[async_trait]
pub trait TransactionManager {
    /// 由具体实现声明自己使用的数据库类型
    type DB: Database;

    /// 开启并返回一个事务句柄
    async fn begin_tx(&self) -> Result<Transaction<'_, Self::DB>, StorageError>;
}
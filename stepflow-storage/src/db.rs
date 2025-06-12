use crate::persistence_manager::PersistenceManager;

// -- SQLite -----------------------------------------------------------
#[cfg(feature = "sqlite")]                   // cargo feature = "sqlite"
pub type DbBackend = sqlx::Sqlite;

// -- Postgres ---------------------------------------------------------
#[cfg(feature = "postgres")]
pub type DbBackend = sqlx::Postgres;

// （可选）MySQL
#[cfg(feature = "mysql")]
pub type DbBackend = sqlx::MySql;

// --------------------------------------------------------------------
// 常用的别名一起给了，省得 everywhere 写长串：
pub type DbPool<'a> = sqlx::Pool<DbBackend>;
pub type Tx<'a>     = sqlx::Transaction<'a, DbBackend>;

use std::sync::Arc;


pub type DynPM = Arc<dyn PersistenceManager<DB = DbBackend> + Send + Sync>;
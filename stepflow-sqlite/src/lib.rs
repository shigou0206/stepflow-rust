pub mod db;
pub mod crud;
pub mod models;
pub mod persistence;
mod storage_manager;


pub use storage_manager::SqliteStorageManager;
pub use models::*;

#[macro_export]
macro_rules! tx_exec {
    ($tx:expr, $fn:ident($($arg:expr),*)) => {
        $fn(&mut *$tx, $($arg),*).await
    };
}
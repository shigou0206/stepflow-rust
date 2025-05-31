use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::env;
use once_cell::sync::Lazy;

// 测试数据库 URL，默认使用内存 SQLite 数据库
static DATABASE_URL: Lazy<String> = Lazy::new(|| {
    env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string())
});

// 创建连接池
pub async fn setup_pool() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&DATABASE_URL)
        .await
        .expect("Failed to create connection pool");

    // 运行迁移脚本（确保数据库表已创建）
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

// 为每个测试开启事务，确保测试隔离
// pub async fn setup_transaction(pool: &Pool<Sqlite>) -> sqlx::Transaction<'_, Sqlite> {
//     pool.begin().await.expect("Failed to begin transaction")
// }
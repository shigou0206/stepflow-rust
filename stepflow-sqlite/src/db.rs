use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use std::str::FromStr;

pub async fn init_db() -> Pool<Sqlite> {
    // ✅ 使用新的数据库文件名：stepflow.db
    let db_path = PathBuf::from("data/stepflow.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    let options = SqliteConnectOptions::from_str(&db_url)
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to create sqlite pool")
}
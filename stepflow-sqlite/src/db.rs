use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub async fn init_db(db_url: &str) -> Pool<Sqlite> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to create pool")
}
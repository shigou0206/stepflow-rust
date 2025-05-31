use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::timer_crud,
    models::timer::{Timer, UpdateTimer},
};
use chrono::NaiveDateTime;

#[derive(Clone)]
pub struct TimerPersistence {
    pool: SqlitePool,
}

impl TimerPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_timer(&self, timer: &Timer) -> Result<(), sqlx::Error> {
        timer_crud::create_timer(&self.pool, timer).await
    }

    pub async fn get_timer(&self, timer_id: &str) -> Result<Option<Timer>, sqlx::Error> {
        timer_crud::get_timer(&self.pool, timer_id).await
    }

    pub async fn update_timer(&self, timer_id: &str, changes: &UpdateTimer) -> Result<(), sqlx::Error> {
        timer_crud::update_timer(&self.pool, timer_id, changes).await
    }

    pub async fn delete_timer(&self, timer_id: &str) -> Result<(), sqlx::Error> {
        timer_crud::delete_timer(&self.pool, timer_id).await
    }

    pub async fn find_timers_before(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<Timer>, sqlx::Error> {
        timer_crud::find_timers_before(&self.pool, before, limit).await
    }
}
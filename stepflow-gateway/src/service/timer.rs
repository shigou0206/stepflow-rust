use chrono::{Utc, DateTime};
use std::sync::Arc;
use stepflow_storage::entities::timer::{StoredTimer, UpdateStoredTimer};
use stepflow_dto::dto::timer::{TimerDto, CreateTimerDto, UpdateTimerDto};
use crate::{
    error::{AppResult, AppError},
    app_state::AppState,
};
use anyhow::anyhow;
use async_trait::async_trait;

#[derive(Clone)]
pub struct TimerSqlxSvc {
    state: Arc<AppState>,
}

impl TimerSqlxSvc {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    fn to_dto(stored: StoredTimer) -> TimerDto {
        TimerDto {
            timer_id: stored.timer_id,
            run_id: stored.run_id,
            shard_id: stored.shard_id,
            fire_at: DateTime::from_naive_utc_and_offset(stored.fire_at, Utc),
            status: stored.status,
            version: stored.version,
            state_name: stored.state_name,
            payload: stored.payload,
            created_at: DateTime::from_naive_utc_and_offset(stored.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(stored.updated_at, Utc),
        }
    }

    fn to_stored_update(dto: UpdateTimerDto) -> UpdateStoredTimer {
        UpdateStoredTimer {
            fire_at: dto.fire_at.map(|dt| dt.naive_utc()),
            status: dto.status,
            version: dto.version,
            payload: dto.payload,
        }
    }
}

#[async_trait]
impl crate::service::TimerService for TimerSqlxSvc {
    async fn create_timer(&self, dto: CreateTimerDto) -> AppResult<TimerDto> {
        let now = Utc::now().naive_utc();

        let stored = StoredTimer {
            timer_id: dto.timer_id,
            run_id: dto.run_id,
            shard_id: dto.shard_id,
            fire_at: dto.fire_at.naive_utc(),
            status: dto.status,
            version: dto.version,
            state_name: dto.state_name,
            payload: dto.payload,
            created_at: now,
            updated_at: now,
        };

        self.state.persist.create_timer(&stored).await
            .map_err(|e| AppError::Anyhow(anyhow!("create_timer failed: {e}")))?;

        Ok(Self::to_dto(stored))
    }

    async fn get_timer(&self, timer_id: &str) -> AppResult<TimerDto> {
        let stored = self.state.persist.get_timer(timer_id).await
            .map_err(|e| AppError::Anyhow(anyhow!("get_timer failed: {e}")))?
            .ok_or(AppError::NotFound)?;
        Ok(Self::to_dto(stored))
    }

    async fn update_timer(&self, timer_id: &str, update: UpdateTimerDto) -> AppResult<TimerDto> {
        let stored_update = Self::to_stored_update(update);

        self.state.persist.update_timer(timer_id, &stored_update).await
            .map_err(|e| AppError::Anyhow(anyhow!("update_timer failed: {e}")))?;

        self.get_timer(timer_id).await
    }

    async fn delete_timer(&self, timer_id: &str) -> AppResult<()> {
        self.state.persist.delete_timer(timer_id).await
            .map_err(|e| AppError::Anyhow(anyhow!("delete_timer failed: {e}")))?;
        Ok(())
    }

    async fn find_timers_before(&self, before: DateTime<Utc>, limit: i64) -> AppResult<Vec<TimerDto>> {
        let naive = before.naive_utc();
        let timers = self.state.persist.find_timers_before(naive, limit).await
            .map_err(|e| AppError::Anyhow(anyhow!("find_timers_before failed: {e}")))?;

        Ok(timers.into_iter().map(Self::to_dto).collect())
    }
}
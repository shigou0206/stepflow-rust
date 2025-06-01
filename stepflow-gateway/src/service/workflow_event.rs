use chrono::Utc;
use std::sync::Arc;
use stepflow_storage::PersistenceManager;
use stepflow_sqlite::models::workflow_event::WorkflowEvent;
use crate::{
    dto::workflow_event::{WorkflowEventDto, RecordEventRequest},
    error::{AppResult, AppError},
};

#[derive(Clone)]
pub struct WorkflowEventSqlxSvc {
    pm: Arc<dyn PersistenceManager>,
}

impl WorkflowEventSqlxSvc {
    pub fn new(pm: Arc<dyn PersistenceManager>) -> Self {
        Self { pm }
    }
}

impl From<WorkflowEvent> for WorkflowEventDto {
    fn from(event: WorkflowEvent) -> Self {
        Self {
            id: event.id,
            run_id: event.run_id,
            shard_id: event.shard_id,
            event_id: event.event_id,
            event_type: event.event_type,
            state_id: event.state_id,
            state_type: event.state_type,
            trace_id: event.trace_id,
            parent_event_id: event.parent_event_id,
            context_version: event.context_version,
            attributes: event.attributes.and_then(|s| serde_json::from_str(&s).ok()),
            attr_version: event.attr_version,
            timestamp: event.timestamp,
            archived: event.archived,
        }
    }
}

impl WorkflowEventSqlxSvc {
    pub async fn list_events(&self, limit: i64, offset: i64) -> AppResult<Vec<WorkflowEventDto>> {
        let events = self.pm.find_events_by_run_id("", limit, offset).await
            .map_err(AppError::Db)?;
        Ok(events.into_iter().map(Into::into).collect())
    }

    pub async fn get_event(&self, id: i64) -> AppResult<Option<WorkflowEventDto>> {
        let event = self.pm.get_event(id).await
            .map_err(AppError::Db)?;
        Ok(event.map(Into::into))
    }

    pub async fn list_events_for_run(&self, run_id: &str, limit: i64, offset: i64) -> AppResult<Vec<WorkflowEventDto>> {
        let events = self.pm.find_events_by_run_id(run_id, limit, offset).await
            .map_err(AppError::Db)?;
        Ok(events.into_iter().map(Into::into).collect())
    }

    pub async fn record_event(&self, req: RecordEventRequest) -> AppResult<WorkflowEventDto> {
        let event = WorkflowEvent {
            id: 0,
            run_id: req.run_id,
            shard_id: req.shard_id,
            event_id: req.event_id,
            event_type: req.event_type,
            state_id: req.state_id,
            state_type: req.state_type,
            trace_id: req.trace_id,
            parent_event_id: req.parent_event_id,
            context_version: req.context_version,
            attributes: req.attributes.map(|v| serde_json::to_string(&v).unwrap()),
            attr_version: 1,
            timestamp: Utc::now().naive_utc(),
            archived: req.archived,
        };

        let id = self.pm.create_event(&event).await
            .map_err(AppError::Db)?;

        let event = self.pm.get_event(id).await
            .map_err(AppError::Db)?
            .ok_or(AppError::NotFound)?;
        Ok(event.into())
    }

    pub async fn archive_event(&self, id: i64) -> AppResult<Option<WorkflowEventDto>> {
        self.pm.archive_event(id).await
            .map_err(AppError::Db)?;

        let event = self.pm.get_event(id).await
            .map_err(AppError::Db)?;
        Ok(event.map(Into::into))
    }

    pub async fn delete_event(&self, id: i64) -> AppResult<()> {
        self.pm.delete_event(id).await
            .map_err(AppError::Db)?;
        Ok(())
    }
} 
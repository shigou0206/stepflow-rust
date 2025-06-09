use chrono::Utc;
use stepflow_storage::db::DynPM;
use stepflow_storage::error::StorageError;
use stepflow_storage::entities::workflow_event::StoredWorkflowEvent;
use stepflow_dto::dto::workflow_event::{WorkflowEventDto, RecordEventRequest};
use crate::{
    error::{AppResult, AppError},
};
use anyhow::Error;

#[derive(Clone)]
pub struct WorkflowEventSqlxSvc {
    pm: DynPM,
}

impl WorkflowEventSqlxSvc {
    pub fn new(pm: DynPM) -> Self {
        Self { pm }
    }
}


impl WorkflowEventSqlxSvc {
    pub async fn list_events(&self, limit: i64, offset: i64) -> AppResult<Vec<WorkflowEventDto>> {
        let events = self.pm.find_events_by_run_id("", limit, offset).await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(events.into_iter().map(Into::into).collect())
    }

    pub async fn get_event(&self, id: i64) -> AppResult<Option<WorkflowEventDto>> {
        let event = self.pm.get_event(id).await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(event.map(Into::into))
    }

    pub async fn list_events_for_run(&self, run_id: &str, limit: i64, offset: i64) -> AppResult<Vec<WorkflowEventDto>> {
        let events = self.pm.find_events_by_run_id(run_id, limit, offset).await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(events.into_iter().map(Into::into).collect())
    }

    pub async fn record_event(&self, req: RecordEventRequest) -> AppResult<WorkflowEventDto> {
        let event = StoredWorkflowEvent {
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
            .map_err(|e: StorageError| Error::new(e))?;

        let event = self.pm.get_event(id).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(event.into())
    }

    pub async fn archive_event(&self, id: i64) -> AppResult<Option<WorkflowEventDto>> {
        self.pm.archive_event(id).await
            .map_err(|e: StorageError| Error::new(e))?;

        let event = self.pm.get_event(id).await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(event.map(Into::into))
    }

    pub async fn delete_event(&self, id: i64) -> AppResult<()> {
        self.pm.delete_event(id).await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(())
    }
} 
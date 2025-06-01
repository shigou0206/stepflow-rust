use chrono::Utc;
use std::sync::Arc;
use stepflow_storage::PersistenceManager;
use stepflow_sqlite::models::workflow_event::WorkflowEvent;
use crate::dto::workflow_event::{WorkflowEventDto, RecordEventRequest};

#[derive(Clone)]
pub struct WorkflowEventService {
    persistence: Arc<dyn PersistenceManager>,
}

impl WorkflowEventService {
    pub fn new(persistence: Arc<dyn PersistenceManager>) -> Self {
        Self { persistence }
    }

    pub async fn list_events(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<WorkflowEventDto>> {
        let events = self.persistence.find_events_by_run_id("", limit, offset).await?;
        Ok(events.into_iter().map(Into::into).collect())
    }

    pub async fn get_event(&self, id: i64) -> anyhow::Result<Option<WorkflowEventDto>> {
        let event = self.persistence.get_event(id).await?;
        Ok(event.map(Into::into))
    }

    pub async fn list_events_for_run(&self, run_id: &str, limit: i64, offset: i64) -> anyhow::Result<Vec<WorkflowEventDto>> {
        let events = self.persistence.find_events_by_run_id(run_id, limit, offset).await?;
        Ok(events.into_iter().map(Into::into).collect())
    }

    pub async fn record_event(&self, req: RecordEventRequest) -> anyhow::Result<WorkflowEventDto> {
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
            attributes: req.attributes,
            attr_version: 1,
            timestamp: Utc::now().naive_utc(),
            archived: req.archived,
        };

        let id = self.persistence.create_event(&event).await?;
        let created = self.persistence.get_event(id).await?
            .ok_or_else(|| anyhow::anyhow!("Failed to get created event"))?;
        
        Ok(created.into())
    }

    pub async fn archive_event(&self, id: i64) -> anyhow::Result<Option<WorkflowEventDto>> {
        self.persistence.archive_event(id).await?;
        let event = self.persistence.get_event(id).await?;
        Ok(event.map(Into::into))
    }

    pub async fn delete_event(&self, id: i64) -> anyhow::Result<()> {
        self.persistence.delete_event(id).await?;
        Ok(())
    }
}

// Model -> DTO 转换实现
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
            attributes: event.attributes,
            attr_version: event.attr_version,
            timestamp: event.timestamp,
            archived: event.archived,
        }
    }
} 
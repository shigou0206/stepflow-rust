use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::workflow_event_crud,
    models::workflow_event::WorkflowEvent,
};

#[derive(Clone)]
pub struct WorkflowEventPersistence {
    pool: SqlitePool,
}

impl WorkflowEventPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_event(&self, event: &WorkflowEvent) -> Result<i64, sqlx::Error> {
        workflow_event_crud::create_event(&self.pool, event).await
    }

    pub async fn get_event(&self, id: i64) -> Result<Option<WorkflowEvent>, sqlx::Error> {
        workflow_event_crud::get_event(&self.pool, id).await
    }

    pub async fn find_events_by_run_id(
        &self,
        run_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkflowEvent>, sqlx::Error> {
        workflow_event_crud::find_events_by_run_id(&self.pool, run_id, limit, offset).await
    }

    pub async fn archive_event(&self, id: i64) -> Result<(), sqlx::Error> {
        workflow_event_crud::archive_event(&self.pool, id).await
    }

    pub async fn delete_event(&self, id: i64) -> Result<(), sqlx::Error> {
        workflow_event_crud::delete_event(&self.pool, id).await
    }

    pub async fn delete_events_by_run_id(&self, run_id: &str) -> Result<u64, sqlx::Error> {
        workflow_event_crud::delete_events_by_run_id(&self.pool, run_id).await
    }
}
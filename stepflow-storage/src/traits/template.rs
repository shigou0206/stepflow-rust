use crate::error::StorageError;
use crate::entities::workflow_template::{StoredWorkflowTemplate, UpdateStoredWorkflowTemplate};

#[async_trait::async_trait]
pub trait TemplateStorage: Send + Sync {
    /// Create a new workflow template
    async fn create_template(&self, tpl: &StoredWorkflowTemplate) -> Result<(), StorageError>;
    
    /// Get a workflow template by template_id
    async fn get_template(&self, template_id: &str) -> Result<Option<StoredWorkflowTemplate>, StorageError>;
    
    /// Find workflow templates with pagination
    async fn find_templates(&self, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowTemplate>, StorageError>;
    
    /// Update a workflow template
    async fn update_template(&self, template_id: &str, changes: &UpdateStoredWorkflowTemplate) -> Result<(), StorageError>;
    
    /// Delete a workflow template
    async fn delete_template(&self, template_id: &str) -> Result<(), StorageError>;
} 
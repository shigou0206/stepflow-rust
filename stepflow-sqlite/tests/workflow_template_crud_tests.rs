mod common;
use common::setup_pool;
use stepflow_sqlite::crud::workflow_template_crud::*;
use stepflow_sqlite::models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate};
use stepflow_sqlite::tx_exec;
use chrono::Utc;
use std::collections::HashSet;

#[tokio::test]
async fn test_create_and_get_template() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let template = WorkflowTemplate {
        template_id: "tpl_001".to_string(),
        name: "Test Workflow".to_string(),
        description: Some("A test workflow template".to_string()),
        dsl_definition: r#"{
            "states": [
                {"name": "start", "type": "start"},
                {"name": "end", "type": "end"}
            ]
        }"#.to_string(),
        version: 1,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_template(&template)).unwrap();
    let fetched = tx_exec!(tx, get_template(&template.template_id)).unwrap().unwrap();

    assert_eq!(fetched.template_id, template.template_id);
    assert_eq!(fetched.name, "Test Workflow");
    assert_eq!(fetched.version, 1);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_templates() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let template1 = WorkflowTemplate {
        template_id: "tpl_002".to_string(),
        name: "First Workflow".to_string(),
        description: Some("First test workflow".to_string()),
        dsl_definition: r#"{"states": []}"#.to_string(),
        version: 1,
        created_at: now,
        updated_at: now,
    };

    let template2 = WorkflowTemplate {
        template_id: "tpl_003".to_string(),
        name: "Second Workflow".to_string(),
        description: Some("Second test workflow".to_string()),
        dsl_definition: r#"{"states": []}"#.to_string(),
        version: 1,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_template(&template1)).unwrap();
    tx_exec!(tx, create_template(&template2)).unwrap();

    let templates = tx_exec!(tx, find_templates(10, 0)).unwrap();
    let template_ids: HashSet<_> = templates.iter().map(|t| &t.template_id).collect();
    let expected_ids: HashSet<_> = vec![&template1.template_id, &template2.template_id].into_iter().collect();

    assert_eq!(templates.len(), 2);
    assert_eq!(template_ids, expected_ids);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_template() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let template = WorkflowTemplate {
        template_id: "tpl_004".to_string(),
        name: "Original Name".to_string(),
        description: Some("Original description".to_string()),
        dsl_definition: r#"{"states": []}"#.to_string(),
        version: 1,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_template(&template)).unwrap();

    let updates = UpdateWorkflowTemplate {
        name: Some("Updated Name".to_string()),
        description: Some("Updated description".to_string()),
        dsl_definition: Some(r#"{"states": [{"name": "new_state"}]}"#.to_string()),
        version: Some(2),
    };

    tx_exec!(tx, update_template(&template.template_id, &updates)).unwrap();
    let updated = tx_exec!(tx, get_template(&template.template_id)).unwrap().unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert_eq!(updated.version, 2);
    assert!(updated.updated_at > template.updated_at);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_template() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let template = WorkflowTemplate {
        template_id: "tpl_005".to_string(),
        name: "To Be Deleted".to_string(),
        description: Some("This template will be deleted".to_string()),
        dsl_definition: r#"{"states": []}"#.to_string(),
        version: 1,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_template(&template)).unwrap();
    tx_exec!(tx, delete_template(&template.template_id)).unwrap();

    let result = tx_exec!(tx, get_template(&template.template_id)).unwrap();
    assert!(result.is_none());
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_template_no_changes() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let template = WorkflowTemplate {
        template_id: "tpl_006".to_string(),
        name: "Test Template".to_string(),
        description: Some("Test description".to_string()),
        dsl_definition: r#"{"states": []}"#.to_string(),
        version: 1,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_template(&template)).unwrap();

    // 测试空更新
    let no_updates = UpdateWorkflowTemplate {
        name: None,
        description: None,
        dsl_definition: None,
        version: None,
    };

    tx_exec!(tx, update_template(&template.template_id, &no_updates)).unwrap();
    let unchanged = tx_exec!(tx, get_template(&template.template_id)).unwrap().unwrap();

    // 验证没有字段被更新
    assert_eq!(unchanged.name, template.name);
    assert_eq!(unchanged.description, template.description);
    assert_eq!(unchanged.version, template.version);
    assert_eq!(unchanged.updated_at, template.updated_at);
    tx.rollback().await.unwrap();
}
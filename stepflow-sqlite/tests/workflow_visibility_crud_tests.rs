mod common;
use common::setup_pool;
use stepflow_sqlite::crud::workflow_visibility_crud::*;
use stepflow_sqlite::models::workflow_visibility::{WorkflowVisibility, UpdateWorkflowVisibility};
use stepflow_sqlite::tx_exec;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_get_visibility() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let vis = WorkflowVisibility {
        run_id: "run_001".to_string(),
        workflow_id: Some("workflow_001".to_string()),
        workflow_type: Some("type_a".to_string()),
        start_time: Some(now),
        close_time: None,
        status: Some("RUNNING".to_string()),
        memo: Some("Initial test".to_string()),
        search_attrs: Some("key1:val1,key2:val2".to_string()),
        version: 1,
    };

    tx_exec!(tx, create_visibility(&vis)).unwrap();
    let fetched = tx_exec!(tx, get_visibility(&vis.run_id)).unwrap().unwrap();

    assert_eq!(fetched.run_id, vis.run_id);
    assert_eq!(fetched.workflow_id, Some("workflow_001".to_string()));
    assert_eq!(fetched.status, Some("RUNNING".to_string()));
    assert_eq!(fetched.memo, Some("Initial test".to_string()));
    assert_eq!(fetched.search_attrs, Some("key1:val1,key2:val2".to_string()));
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_visibilities_by_status() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let vis1 = WorkflowVisibility {
        run_id: "run_002".to_string(),
        workflow_id: Some("workflow_002".to_string()),
        workflow_type: Some("type_b".to_string()),
        start_time: Some(now),
        close_time: None,
        status: Some("RUNNING".to_string()),
        memo: None,
        search_attrs: None,
        version: 1,
    };

    let vis2 = WorkflowVisibility {
        run_id: "run_003".to_string(),
        workflow_id: Some("workflow_003".to_string()),
        workflow_type: Some("type_b".to_string()),
        start_time: Some(now),
        close_time: None,
        status: Some("RUNNING".to_string()),
        memo: None,
        search_attrs: None,
        version: 1,
    };

    tx_exec!(tx, create_visibility(&vis1)).unwrap();
    tx_exec!(tx, create_visibility(&vis2)).unwrap();

    let visibilities = tx_exec!(tx, find_visibilities_by_status("RUNNING", 10, 0)).unwrap();
    assert_eq!(visibilities.len(), 2);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_visibility() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let vis = WorkflowVisibility {
        run_id: "run_004".to_string(),
        workflow_id: Some("workflow_004".to_string()),
        workflow_type: Some("type_c".to_string()),
        start_time: Some(now),
        close_time: None,
        status: Some("RUNNING".to_string()),
        memo: None,
        search_attrs: None,
        version: 1,
    };

    tx_exec!(tx, create_visibility(&vis)).unwrap();

    let updates = UpdateWorkflowVisibility {
        status: Some("COMPLETED".to_string()),
        close_time: Some(Utc::now().naive_utc()),
        memo: Some("Test complete".to_string()),
        ..Default::default()
    };

    tx_exec!(tx, update_visibility(&vis.run_id, &updates)).unwrap();
    let updated = tx_exec!(tx, get_visibility(&vis.run_id)).unwrap().unwrap();

    assert_eq!(updated.status, Some("COMPLETED".to_string()));
    assert_eq!(updated.memo, Some("Test complete".to_string()));
    assert!(updated.close_time.is_some());
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_visibility() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let vis = WorkflowVisibility {
        run_id: "run_005".to_string(),
        workflow_id: Some("workflow_005".to_string()),
        workflow_type: Some("type_d".to_string()),
        start_time: Some(now),
        close_time: None,
        status: Some("FAILED".to_string()),
        memo: None,
        search_attrs: None,
        version: 1,
    };

    tx_exec!(tx, create_visibility(&vis)).unwrap();
    tx_exec!(tx, delete_visibility(&vis.run_id)).unwrap();

    let result = tx_exec!(tx, get_visibility(&vis.run_id)).unwrap();
    assert!(result.is_none());
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_visibility_no_changes() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let vis = WorkflowVisibility {
        run_id: "run_006".to_string(),
        workflow_id: Some("workflow_006".to_string()),
        workflow_type: Some("type_e".to_string()),
        start_time: Some(now),
        close_time: None,
        status: Some("RUNNING".to_string()),
        memo: None,
        search_attrs: None,
        version: 1,
    };

    tx_exec!(tx, create_visibility(&vis)).unwrap();

    let no_updates = UpdateWorkflowVisibility::default();
    tx_exec!(tx, update_visibility(&vis.run_id, &no_updates)).unwrap();
    let unchanged = tx_exec!(tx, get_visibility(&vis.run_id)).unwrap().unwrap();

    assert_eq!(unchanged.status, vis.status);
    assert_eq!(unchanged.version, vis.version);
    tx.rollback().await.unwrap();
}
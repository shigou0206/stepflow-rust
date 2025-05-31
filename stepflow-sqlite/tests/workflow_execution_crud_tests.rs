mod common;
use common::setup_pool;
use stepflow_sqlite::crud::workflow_execution_crud::*;
use stepflow_sqlite::models::workflow_execution::{WorkflowExecution, UpdateWorkflowExecution};
use stepflow_sqlite::tx_exec;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_get_execution() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let exec = WorkflowExecution {
        run_id: "run_001".to_string(),
        workflow_id: Some("wf_001".to_string()),
        shard_id: 1,
        template_id: Some("tpl_001".to_string()),
        mode: "sync".to_string(),
        current_state_name: Some("initial".to_string()),
        status: "running".to_string(),
        workflow_type: "test_workflow".to_string(),
        input: Some("{}".to_string()),
        input_version: 1,
        result: None,
        result_version: 1,
        start_time: Utc::now().naive_utc(),
        close_time: None,
        current_event_id: 1,
        memo: None,
        search_attrs: None,
        context_snapshot: None,
        version: 1,
    };

    tx_exec!(tx, create_execution(&exec)).unwrap();
    let fetched = tx_exec!(tx, get_execution(&exec.run_id)).unwrap().unwrap();

    assert_eq!(fetched.run_id, exec.run_id);
    assert_eq!(fetched.status, "running");
    assert_eq!(fetched.workflow_type, "test_workflow");
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_executions() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let exec1 = WorkflowExecution {
        run_id: "run_002".to_string(),
        workflow_id: Some("wf_002".to_string()),
        shard_id: 1,
        template_id: Some("tpl_001".to_string()),
        mode: "sync".to_string(),
        current_state_name: Some("initial".to_string()),
        status: "running".to_string(),
        workflow_type: "test_workflow".to_string(),
        input: Some("{}".to_string()),
        input_version: 1,
        result: None,
        result_version: 1,
        start_time: Utc::now().naive_utc(),
        close_time: None,
        current_event_id: 1,
        memo: None,
        search_attrs: None,
        context_snapshot: None,
        version: 1,
    };

    let exec2 = WorkflowExecution {
        run_id: "run_003".to_string(),
        workflow_id: Some("wf_003".to_string()),
        shard_id: 1,
        template_id: Some("tpl_001".to_string()),
        mode: "sync".to_string(),
        current_state_name: Some("completed".to_string()),
        status: "completed".to_string(),
        workflow_type: "test_workflow".to_string(),
        input: Some("{}".to_string()),
        input_version: 1,
        result: Some("{}".to_string()),
        result_version: 1,
        start_time: Utc::now().naive_utc(),
        close_time: Some(Utc::now().naive_utc()),
        current_event_id: 2,
        memo: None,
        search_attrs: None,
        context_snapshot: None,
        version: 2,
    };

    tx_exec!(tx, create_execution(&exec1)).unwrap();
    tx_exec!(tx, create_execution(&exec2)).unwrap();

    // Test find all executions
    let executions = tx_exec!(tx, find_executions(10, 0)).unwrap();
    assert_eq!(executions.len(), 2);

    // Test find by status
    let running = tx_exec!(tx, find_executions_by_status("running", 10, 0)).unwrap();
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].run_id, "run_002");

    let completed = tx_exec!(tx, find_executions_by_status("completed", 10, 0)).unwrap();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].run_id, "run_003");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_execution() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let exec = WorkflowExecution {
        run_id: "run_004".to_string(),
        workflow_id: Some("wf_004".to_string()),
        shard_id: 1,
        template_id: Some("tpl_001".to_string()),
        mode: "sync".to_string(),
        current_state_name: Some("initial".to_string()),
        status: "running".to_string(),
        workflow_type: "test_workflow".to_string(),
        input: Some("{}".to_string()),
        input_version: 1,
        result: None,
        result_version: 1,
        start_time: Utc::now().naive_utc(),
        close_time: None,
        current_event_id: 1,
        memo: None,
        search_attrs: None,
        context_snapshot: None,
        version: 1,
    };

    tx_exec!(tx, create_execution(&exec)).unwrap();

    let updates = UpdateWorkflowExecution {
        status: Some("completed".to_string()),
        current_state_name: Some("final".to_string()),
        result: Some("{}".to_string()),
        close_time: Some(Utc::now().naive_utc()),
        version: Some(2),
        ..Default::default()
    };

    tx_exec!(tx, update_execution(&exec.run_id, &updates)).unwrap();
    let updated = tx_exec!(tx, get_execution(&exec.run_id)).unwrap().unwrap();

    assert_eq!(updated.status, "completed");
    assert_eq!(updated.current_state_name, Some("final".to_string()));
    assert_eq!(updated.version, 2);
    assert!(updated.close_time.is_some());
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_execution() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let exec = WorkflowExecution {
        run_id: "run_005".to_string(),
        workflow_id: Some("wf_005".to_string()),
        shard_id: 1,
        template_id: Some("tpl_001".to_string()),
        mode: "sync".to_string(),
        current_state_name: Some("initial".to_string()),
        status: "running".to_string(),
        workflow_type: "test_workflow".to_string(),
        input: Some("{}".to_string()),
        input_version: 1,
        result: None,
        result_version: 1,
        start_time: Utc::now().naive_utc(),
        close_time: None,
        current_event_id: 1,
        memo: None,
        search_attrs: None,
        context_snapshot: None,
        version: 1,
    };

    tx_exec!(tx, create_execution(&exec)).unwrap();
    tx_exec!(tx, delete_execution(&exec.run_id)).unwrap();

    let result = tx_exec!(tx, get_execution(&exec.run_id)).unwrap();
    assert!(result.is_none());
    tx.rollback().await.unwrap();
} 
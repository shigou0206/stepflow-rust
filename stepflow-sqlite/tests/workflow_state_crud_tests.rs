mod common;
use common::setup_pool;
use stepflow_sqlite::crud::workflow_state_crud::*;
use stepflow_sqlite::models::workflow_state::{WorkflowState, UpdateWorkflowState};
use stepflow_sqlite::tx_exec;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_get_state() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let state = WorkflowState {
        state_id: "state_001".to_string(),
        run_id: "run_001".to_string(),
        shard_id: 1,
        state_name: "initial".to_string(),
        state_type: "start".to_string(),
        status: "running".to_string(),
        input: Some("{}".to_string()),
        output: None,
        error: None,
        error_details: None,
        started_at: Some(now),
        completed_at: None,
        created_at: now,
        updated_at: now,
        version: 1,
    };

    tx_exec!(tx, create_state(&state)).unwrap();
    let fetched = tx_exec!(tx, get_state(&state.state_id)).unwrap().unwrap();

    assert_eq!(fetched.state_id, state.state_id);
    assert_eq!(fetched.state_name, "initial");
    assert_eq!(fetched.status, "running");
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_states_by_run_id() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let state1 = WorkflowState {
        state_id: "state_002".to_string(),
        run_id: "run_002".to_string(),
        shard_id: 1,
        state_name: "initial".to_string(),
        state_type: "start".to_string(),
        status: "completed".to_string(),
        input: Some("{}".to_string()),
        output: Some("{}".to_string()),
        error: None,
        error_details: None,
        started_at: Some(now),
        completed_at: Some(now),
        created_at: now,
        updated_at: now,
        version: 1,
    };

    let state2 = WorkflowState {
        state_id: "state_003".to_string(),
        run_id: "run_002".to_string(),
        shard_id: 1,
        state_name: "task1".to_string(),
        state_type: "task".to_string(),
        status: "running".to_string(),
        input: Some("{}".to_string()),
        output: None,
        error: None,
        error_details: None,
        started_at: Some(now),
        completed_at: None,
        created_at: now,
        updated_at: now,
        version: 1,
    };

    tx_exec!(tx, create_state(&state1)).unwrap();
    tx_exec!(tx, create_state(&state2)).unwrap();

    let states = tx_exec!(tx, find_states_by_run_id("run_002", 10, 0)).unwrap();

    assert_eq!(states.len(), 2);
    assert_eq!(states[0].state_name, "initial");
    assert_eq!(states[1].state_name, "task1");
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_state() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let state = WorkflowState {
        state_id: "state_004".to_string(),
        run_id: "run_003".to_string(),
        shard_id: 1,
        state_name: "task1".to_string(),
        state_type: "task".to_string(),
        status: "running".to_string(),
        input: Some("{}".to_string()),
        output: None,
        error: None,
        error_details: None,
        started_at: Some(now),
        completed_at: None,
        created_at: now,
        updated_at: now,
        version: 1,
    };

    tx_exec!(tx, create_state(&state)).unwrap();

    let updates = UpdateWorkflowState {
        status: Some("completed".to_string()),
        output: Some("{\"result\": \"success\"}".to_string()),
        completed_at: Some(Utc::now().naive_utc()),
        version: Some(2),
        ..Default::default()
    };

    tx_exec!(tx, update_state(&state.state_id, &updates)).unwrap();
    let updated = tx_exec!(tx, get_state(&state.state_id)).unwrap().unwrap();

    assert_eq!(updated.status, "completed");
    assert_eq!(updated.output, Some("{\"result\": \"success\"}".to_string()));
    assert!(updated.completed_at.is_some());
    assert_eq!(updated.version, 2);
    assert!(updated.updated_at > state.updated_at);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_state() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let state = WorkflowState {
        state_id: "state_005".to_string(),
        run_id: "run_004".to_string(),
        shard_id: 1,
        state_name: "task1".to_string(),
        state_type: "task".to_string(),
        status: "running".to_string(),
        input: Some("{}".to_string()),
        output: None,
        error: None,
        error_details: None,
        started_at: Some(now),
        completed_at: None,
        created_at: now,
        updated_at: now,
        version: 1,
    };

    tx_exec!(tx, create_state(&state)).unwrap();
    tx_exec!(tx, delete_state(&state.state_id)).unwrap();

    let result = tx_exec!(tx, get_state(&state.state_id)).unwrap();
    assert!(result.is_none());
    tx.rollback().await.unwrap();
} 
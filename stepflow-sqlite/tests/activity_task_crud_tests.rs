mod common;

use common::setup_pool;
use stepflow_sqlite::crud::activity_task_crud::*;
use stepflow_sqlite::models::activity_task::{ActivityTask, UpdateActivityTask};
use stepflow_sqlite::tx_exec;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_get_activity_task() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let task = ActivityTask {
        task_token: "task_001".to_string(),
        run_id: "run_001".to_string(),
        shard_id: 1,
        seq: 1,
        activity_type: "test_type".to_string(),
        state_name: Some("initial".to_string()),
        input: Some("{}".to_string()),
        result: None,
        status: "scheduled".to_string(),
        error: None,
        error_details: None,
        attempt: 0,
        max_attempts: 3,
        heartbeat_at: None,
        scheduled_at: Utc::now().naive_utc(),
        started_at: None,
        completed_at: None,
        timeout_seconds: Some(60),
        retry_policy: None,
        version: 1,
    };

    tx_exec!(tx, create_task(&task)).unwrap();
    let fetched = tx_exec!(tx, get_task(&task.task_token)).unwrap().unwrap();

    assert_eq!(fetched.task_token, task.task_token);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_activity_task() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let task = ActivityTask {
        task_token: "task_002".to_string(),
        run_id: "run_002".to_string(),
        shard_id: 1,
        seq: 2,
        activity_type: "test_type".to_string(),
        state_name: Some("initial".to_string()),
        input: Some("{\"value\": 1}".to_string()),
        result: None,
        status: "scheduled".to_string(),
        error: None,
        error_details: None,
        attempt: 0,
        max_attempts: 3,
        heartbeat_at: None,
        scheduled_at: Utc::now().naive_utc(),
        started_at: None,
        completed_at: None,
        timeout_seconds: Some(120),
        retry_policy: None,
        version: 1,
    };

    tx_exec!(tx, create_task(&task)).unwrap();

    let updates = UpdateActivityTask {
        status: Some("completed".to_string()),
        result: Some("{\"value\": 2}".to_string()),
        version: Some(2),
        ..Default::default()
    };

    tx_exec!(tx, update_task(&task.task_token, &updates)).unwrap();
    let updated = tx_exec!(tx, get_task(&task.task_token)).unwrap().unwrap();

    assert_eq!(updated.status, "completed");
    assert_eq!(updated.result.unwrap(), "{\"value\": 2}");
    assert_eq!(updated.version, 2);

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_activity_tasks_by_status() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let task = ActivityTask {
        task_token: "task_003".to_string(),
        run_id: "run_003".to_string(),
        shard_id: 1,
        seq: 3,
        activity_type: "test_type".to_string(),
        state_name: Some("initial".to_string()),
        input: Some("{}".to_string()),
        result: None,
        status: "scheduled".to_string(),
        error: None,
        error_details: None,
        attempt: 0,
        max_attempts: 3,
        heartbeat_at: None,
        scheduled_at: Utc::now().naive_utc(),
        started_at: None,
        completed_at: None,
        timeout_seconds: None,
        retry_policy: None,
        version: 1,
    };

    tx_exec!(tx, create_task(&task)).unwrap();
    let tasks = tx_exec!(tx, find_tasks_by_status("scheduled", 10, 0)).unwrap();

    assert!(tasks.iter().any(|t| t.task_token == task.task_token));

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_activity_task() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let task = ActivityTask {
        task_token: "task_004".to_string(),
        run_id: "run_004".to_string(),
        shard_id: 1,
        seq: 4,
        activity_type: "test_type".to_string(),
        state_name: Some("initial".to_string()),
        input: Some("{}".to_string()),
        result: None,
        status: "scheduled".to_string(),
        error: None,
        error_details: None,
        attempt: 0,
        max_attempts: 3,
        heartbeat_at: None,
        scheduled_at: Utc::now().naive_utc(),
        started_at: None,
        completed_at: None,
        timeout_seconds: None,
        retry_policy: None,
        version: 1,
    };

    tx_exec!(tx, create_task(&task)).unwrap();
    tx_exec!(tx, delete_task(&task.task_token)).unwrap();
    let fetched = tx_exec!(tx, get_task(&task.task_token)).unwrap();

    assert!(fetched.is_none());

    tx.rollback().await.unwrap();
}
mod common;

use common::setup_pool;
use stepflow_sqlite::crud::queue_task_crud::*;
use stepflow_sqlite::models::queue_task::{QueueTask, UpdateQueueTask};
use stepflow_sqlite::tx_exec;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_get_queue_task() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();

    let task = QueueTask {
        task_id: "task_q_001".to_string(),
        run_id: "run_q_001".to_string(),
        state_name: "initial".to_string(),
        task_payload: Some(r#"{"key": "value"}"#.to_string()),
        status: "queued".to_string(),
        attempts: 0,
        max_attempts: 3,
        error_message: None,
        last_error_at: None,
        next_retry_at: None,
        queued_at: now,
        processing_at: None,
        completed_at: None,
        failed_at: None,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_task(&task)).unwrap();
    let fetched = tx_exec!(tx, get_task(&task.task_id)).unwrap().unwrap();

    assert_eq!(fetched.task_id, task.task_id);
    assert_eq!(fetched.status, "queued");
    assert_eq!(fetched.task_payload, Some(r#"{"key": "value"}"#.to_string()));

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_queue_task() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();

    let task = QueueTask {
        task_id: "task_q_002".to_string(),
        run_id: "run_q_002".to_string(),
        state_name: "initial".to_string(),
        task_payload: None,
        status: "queued".to_string(),
        attempts: 0,
        max_attempts: 3,
        error_message: None,
        last_error_at: None,
        next_retry_at: None,
        queued_at: now,
        processing_at: None,
        completed_at: None,
        failed_at: None,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_task(&task)).unwrap();

    let processing_time = Utc::now().naive_utc();
    let updates = UpdateQueueTask {
        status: Some("processing".to_string()),
        attempts: Some(1),
        processing_at: Some(processing_time),
        ..Default::default()
    };

    tx_exec!(tx, update_task(&task.task_id, &updates)).unwrap();
    let updated = tx_exec!(tx, get_task(&task.task_id)).unwrap().unwrap();

    assert_eq!(updated.status, "processing");
    assert_eq!(updated.attempts, 1);
    assert_eq!(updated.processing_at, Some(processing_time));

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_queue_tasks_by_status() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();

    let task1 = QueueTask {
        task_id: "task_q_003".to_string(),
        run_id: "run_q_003".to_string(),
        state_name: "state1".to_string(),
        task_payload: None,
        status: "queued".to_string(),
        attempts: 0,
        max_attempts: 3,
        error_message: None,
        last_error_at: None,
        next_retry_at: None,
        queued_at: now,
        processing_at: None,
        completed_at: None,
        failed_at: None,
        created_at: now,
        updated_at: now,
    };

    let task2 = QueueTask {
        task_id: "task_q_004".to_string(),
        run_id: "run_q_004".to_string(),
        state_name: "state2".to_string(),
        task_payload: None,
        status: "queued".to_string(),
        attempts: 0,
        max_attempts: 3,
        error_message: None,
        last_error_at: None,
        next_retry_at: None,
        queued_at: now,
        processing_at: None,
        completed_at: None,
        failed_at: None,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_task(&task1)).unwrap();
    tx_exec!(tx, create_task(&task2)).unwrap();

    let tasks = tx_exec!(tx, find_tasks_by_status("queued", 10, 0)).unwrap();

    assert!(tasks.iter().any(|t| t.task_id == task1.task_id));
    assert!(tasks.iter().any(|t| t.task_id == task2.task_id));

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_queue_task() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();

    let task = QueueTask {
        task_id: "task_q_005".to_string(),
        run_id: "run_q_005".to_string(),
        state_name: "state3".to_string(),
        task_payload: None,
        status: "queued".to_string(),
        attempts: 0,
        max_attempts: 3,
        error_message: None,
        last_error_at: None,
        next_retry_at: None,
        queued_at: now,
        processing_at: None,
        completed_at: None,
        failed_at: None,
        created_at: now,
        updated_at: now,
    };

    tx_exec!(tx, create_task(&task)).unwrap();
    tx_exec!(tx, delete_task(&task.task_id)).unwrap();

    let fetched = tx_exec!(tx, get_task(&task.task_id)).unwrap();
    assert!(fetched.is_none());

    tx.rollback().await.unwrap();
}
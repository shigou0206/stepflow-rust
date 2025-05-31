mod common;
use common::setup_pool;
use stepflow_sqlite::crud::timer_crud::*;
use stepflow_sqlite::models::timer::{Timer, UpdateTimer};
use stepflow_sqlite::tx_exec;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_get_timer() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let timer = Timer {
        timer_id: "timer_001".to_string(),
        run_id: "run_001".to_string(),
        shard_id: 1,
        fire_at: Utc::now().naive_utc(),
        status: "scheduled".to_string(),
        version: 1,
        state_name: "initial".to_string(),
    };

    tx_exec!(tx, create_timer(&timer)).unwrap();
    let fetched = tx_exec!(tx, get_timer(&timer.timer_id)).unwrap().unwrap();

    assert_eq!(fetched.timer_id, timer.timer_id);
    assert_eq!(fetched.status, "scheduled");
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_update_timer() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let timer = Timer {
        timer_id: "timer_002".to_string(),
        run_id: "run_002".to_string(),
        shard_id: 2,
        fire_at: Utc::now().naive_utc(),
        status: "scheduled".to_string(),
        version: 1,
        state_name: "initial".to_string(),
    };

    tx_exec!(tx, create_timer(&timer)).unwrap();

    let updates = UpdateTimer {
        status: Some("completed".to_string()),
        version: Some(2),
        ..Default::default()
    };

    tx_exec!(tx, update_timer(&timer.timer_id, &updates)).unwrap();
    let updated = tx_exec!(tx, get_timer(&timer.timer_id)).unwrap().unwrap();

    assert_eq!(updated.status, "completed");
    assert_eq!(updated.version, 2);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_timers_before() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let now = Utc::now().naive_utc();
    let timer = Timer {
        timer_id: "timer_003".to_string(),
        run_id: "run_003".to_string(),
        shard_id: 3,
        fire_at: now,
        status: "scheduled".to_string(),
        version: 1,
        state_name: "initial".to_string(),
    };

    tx_exec!(tx, create_timer(&timer)).unwrap();
    let timers = tx_exec!(tx, find_timers_before(now, 10)).unwrap();

    assert!(!timers.is_empty());
    assert!(timers.iter().any(|t| t.timer_id == timer.timer_id));
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_timer() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let timer = Timer {
        timer_id: "timer_004".to_string(),
        run_id: "run_004".to_string(),
        shard_id: 4,
        fire_at: Utc::now().naive_utc(),
        status: "scheduled".to_string(),
        version: 1,
        state_name: "initial".to_string(),
    };

    tx_exec!(tx, create_timer(&timer)).unwrap();
    tx_exec!(tx, delete_timer(&timer.timer_id)).unwrap();

    let result = tx_exec!(tx, get_timer(&timer.timer_id)).unwrap();
    assert!(result.is_none());
    tx.rollback().await.unwrap();
} 
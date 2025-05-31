mod common;
use common::setup_pool;
use stepflow_sqlite::crud::workflow_event_crud::*;
use stepflow_sqlite::models::workflow_event::WorkflowEvent;
use stepflow_sqlite::tx_exec;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_get_event() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let event = WorkflowEvent {
        id: 0, // 会被数据库自动生成
        run_id: "run_001".to_string(),
        shard_id: 1,
        event_id: 1,
        event_type: "workflow_started".to_string(),
        state_id: Some("state_001".to_string()),
        state_type: Some("initial".to_string()),
        trace_id: Some("trace_001".to_string()),
        parent_event_id: None,
        context_version: Some(1),
        attributes: Some("{}".to_string()),
        attr_version: 1,
        timestamp: Utc::now().naive_utc(),
        archived: false,
    };

    let id = tx_exec!(tx, create_event(&event)).unwrap();
    let fetched = tx_exec!(tx, get_event(id)).unwrap().unwrap();

    assert_eq!(fetched.run_id, event.run_id);
    assert_eq!(fetched.event_type, event.event_type);
    assert_eq!(fetched.archived, false);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_find_events_by_run_id() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let event1 = WorkflowEvent {
        id: 0,
        run_id: "run_002".to_string(),
        shard_id: 1,
        event_id: 1,
        event_type: "workflow_started".to_string(),
        state_id: Some("state_001".to_string()),
        state_type: Some("initial".to_string()),
        trace_id: Some("trace_001".to_string()),
        parent_event_id: None,
        context_version: Some(1),
        attributes: Some("{}".to_string()),
        attr_version: 1,
        timestamp: Utc::now().naive_utc(),
        archived: false,
    };

    let event2 = WorkflowEvent {
        id: 0,
        run_id: "run_002".to_string(),
        shard_id: 1,
        event_id: 2,
        event_type: "task_completed".to_string(),
        state_id: Some("state_002".to_string()),
        state_type: Some("task".to_string()),
        trace_id: Some("trace_001".to_string()),
        parent_event_id: Some(1),
        context_version: Some(1),
        attributes: Some("{}".to_string()),
        attr_version: 1,
        timestamp: Utc::now().naive_utc(),
        archived: false,
    };

    tx_exec!(tx, create_event(&event1)).unwrap();
    tx_exec!(tx, create_event(&event2)).unwrap();

    let events = tx_exec!(tx, find_events_by_run_id("run_002", 10, 0)).unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_id, 1);
    assert_eq!(events[1].event_id, 2);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_archive_event() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let event = WorkflowEvent {
        id: 0,
        run_id: "run_003".to_string(),
        shard_id: 1,
        event_id: 1,
        event_type: "workflow_started".to_string(),
        state_id: Some("state_001".to_string()),
        state_type: Some("initial".to_string()),
        trace_id: Some("trace_001".to_string()),
        parent_event_id: None,
        context_version: Some(1),
        attributes: Some("{}".to_string()),
        attr_version: 1,
        timestamp: Utc::now().naive_utc(),
        archived: false,
    };

    let id = tx_exec!(tx, create_event(&event)).unwrap();
    tx_exec!(tx, archive_event(id)).unwrap();

    let archived = tx_exec!(tx, get_event(id)).unwrap().unwrap();
    assert!(archived.archived);
    assert!(archived.timestamp > event.timestamp);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_events() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let event = WorkflowEvent {
        id: 0,
        run_id: "run_004".to_string(),
        shard_id: 1,
        event_id: 1,
        event_type: "workflow_started".to_string(),
        state_id: Some("state_001".to_string()),
        state_type: Some("initial".to_string()),
        trace_id: Some("trace_001".to_string()),
        parent_event_id: None,
        context_version: Some(1),
        attributes: Some("{}".to_string()),
        attr_version: 1,
        timestamp: Utc::now().naive_utc(),
        archived: false,
    };

    let id = tx_exec!(tx, create_event(&event)).unwrap();
    
    // Test single event deletion
    tx_exec!(tx, delete_event(id)).unwrap();
    let deleted = tx_exec!(tx, get_event(id)).unwrap();
    assert!(deleted.is_none());

    // Test bulk deletion by run_id
    tx_exec!(tx, create_event(&event)).unwrap();
    let deleted_count = tx_exec!(tx, delete_events_by_run_id("run_004")).unwrap();
    assert_eq!(deleted_count, 1);

    let events = tx_exec!(tx, find_events_by_run_id("run_004", 10, 0)).unwrap();
    assert!(events.is_empty());
    
    tx.rollback().await.unwrap();
} 
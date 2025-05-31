use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::{PersistenceManager, PersistenceManagerImpl};
use stepflow_sqlite::models::workflow_event::WorkflowEvent;
use serde_json::json;
use chrono::Utc;

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_workflow_event_persistence_full() {
    // 初始化数据库
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    // 创建持久化管理器实例
    let persistence = PersistenceManagerImpl::new(pool.clone());

    // 1. 测试创建事件
    let event = WorkflowEvent {
        id: 0, // 自增，传0或默认值即可
        run_id: "run_001".to_owned(),
        shard_id: 1,
        event_id: 100,
        event_type: "StateEntered".to_owned(),
        state_id: Some("state_1".to_owned()),
        state_type: Some("Task".to_owned()),
        trace_id: None,
        parent_event_id: None,
        context_version: Some(1),
        attributes: Some(json!({"key": "value"}).to_string()),
        attr_version: 1,
        timestamp: Utc::now().naive_utc(),
        archived: false,
    };

    let event_id = persistence.create_event(&event).await.unwrap();
    assert!(event_id > 0);

    // 2. 测试单个事件查询
    let fetched_event = persistence.get_event(event_id).await.unwrap();
    assert!(fetched_event.is_some());
    assert_eq!(fetched_event.unwrap().run_id, "run_001");

    // 3. 测试分页查询事件
    let events = persistence.find_events_by_run_id("run_001", 10, 0).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "StateEntered");

    // 4. 测试事件归档标记
    persistence.archive_event(event_id).await.unwrap();
    let archived_event = persistence.get_event(event_id).await.unwrap().unwrap();
    assert!(archived_event.archived);

    // 5. 测试删除单个事件
    persistence.delete_event(event_id).await.unwrap();
    let deleted_event = persistence.get_event(event_id).await.unwrap();
    assert!(deleted_event.is_none());

    // 6. 测试批量删除
    // 先插入多个事件
    for i in 0..5 {
        let event = WorkflowEvent {
            id: 0,
            run_id: "run_001".to_owned(),
            shard_id: 1,
            event_id: 200 + i,
            event_type: "StateExited".to_owned(),
            state_id: Some(format!("state_{}", i)),
            state_type: Some("Task".to_owned()),
            trace_id: None,
            parent_event_id: None,
            context_version: Some(1),
            attributes: None,
            attr_version: 1,
            timestamp: Utc::now().naive_utc(),
            archived: false,
        };
        persistence.create_event(&event).await.unwrap();
    }

    let events_before_delete = persistence.find_events_by_run_id("run_001", 10, 0).await.unwrap();
    assert_eq!(events_before_delete.len(), 5);

    let deleted_count = persistence.delete_events_by_run_id("run_001").await.unwrap();
    assert_eq!(deleted_count, 5);

    let events_after_delete = persistence.find_events_by_run_id("run_001", 10, 0).await.unwrap();
    assert_eq!(events_after_delete.len(), 0);
}
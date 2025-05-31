use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::{PersistenceManager, PersistenceManagerImpl};
use stepflow_sqlite::models::activity_task::{ActivityTask, UpdateActivityTask};
use serde_json::json;
use chrono::Utc;

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_activity_task_persistence_full() {
    // 初始化数据库
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    // 创建持久化管理器实例
    let persistence = PersistenceManagerImpl::new(pool.clone());

    // 1. 测试创建任务
    let task = ActivityTask {
        task_token: "task_001".to_owned(),
        run_id: "run_001".to_owned(),
        shard_id: 1,
        seq: 1,
        activity_type: "Email".to_owned(),
        state_name: Some("Pending".to_owned()),
        input: Some(json!({"recipient": "user@example.com"}).to_string()),
        result: None,
        status: "scheduled".to_owned(),
        error: None,
        error_details: None,
        attempt: 0,
        max_attempts: 3,
        heartbeat_at: None,
        scheduled_at: Utc::now().naive_utc(),
        started_at: None,
        completed_at: None,
        timeout_seconds: Some(300),
        retry_policy: None,
        version: 1,
    };

    persistence.create_task(&task).await.unwrap();

    // 2. 测试根据task_token查询单个任务
    let fetched_task = persistence.get_task("task_001").await.unwrap();
    assert!(fetched_task.is_some());
    assert_eq!(fetched_task.unwrap().status, "scheduled");

    // 3. 测试分页查询任务
    let tasks = persistence.find_tasks_by_status("scheduled", 10, 0).await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_token, "task_001");

    // 4. 测试更新任务
    let changes = UpdateActivityTask {
        status: Some("completed".to_owned()),
        result: Some(json!({"success": true}).to_string()),
        completed_at: Some(Utc::now().naive_utc()),
        ..Default::default()
    };
    persistence.update_task("task_001", &changes).await.unwrap();

    let updated_task = persistence.get_task("task_001").await.unwrap().unwrap();
    assert_eq!(updated_task.status, "completed");
    assert!(updated_task.result.is_some());

    // 5. 测试删除任务
    persistence.delete_task("task_001").await.unwrap();
    let deleted_task = persistence.get_task("task_001").await.unwrap();
    assert!(deleted_task.is_none());
}
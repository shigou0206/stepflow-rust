use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::QueueTaskPersistence;
use stepflow_sqlite::models::queue_task::{QueueTask, UpdateQueueTask};
use chrono::Utc;

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_queue_task_persistence_full() {
    // 初始化数据库（内存数据库）
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    // 创建持久化管理器实例
    let persistence = QueueTaskPersistence::new(pool.clone());

    let now = Utc::now().naive_utc();

    // 1. 测试创建任务 (新增完整字段)
    let task = QueueTask {
        task_id: "queue_task_001".to_owned(),
        run_id: "run_queue_001".to_owned(),
        state_name: "PendingState".to_owned(),
        task_payload: Some(r#"{"action": "email"}"#.to_owned()),
        status: "queued".to_owned(),
        attempts: 0,
        max_attempts: 5,
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

    persistence.create_task(&task).await.unwrap();

    // 2. 测试根据 task_id 查询单个任务 (验证字段)
    let fetched_task = persistence.get_task("queue_task_001").await.unwrap();
    assert!(fetched_task.is_some());
    let fetched_task = fetched_task.unwrap();
    assert_eq!(fetched_task.status, "queued");
    assert_eq!(fetched_task.task_payload, Some(r#"{"action": "email"}"#.to_owned()));
    assert_eq!(fetched_task.attempts, 0);

    // 3. 测试根据状态分页查询任务
    let tasks = persistence.find_tasks_by_status("queued", 10, 0).await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, "queue_task_001");

    // 4. 测试更新任务状态 (新增字段测试)
    let processing_time = Utc::now().naive_utc();
    let changes = UpdateQueueTask {
        status: Some("processing".to_owned()),
        attempts: Some(1),
        processing_at: Some(processing_time),
        updated_at: Some(Utc::now().naive_utc()),
        ..Default::default()
    };
    persistence.update_task("queue_task_001", &changes).await.unwrap();

    let updated_task = persistence.get_task("queue_task_001").await.unwrap().unwrap();
    assert_eq!(updated_task.status, "processing");
    assert_eq!(updated_task.attempts, 1);
    assert_eq!(updated_task.processing_at, Some(processing_time));

    // 5. 测试删除任务
    persistence.delete_task("queue_task_001").await.unwrap();
    let deleted_task = persistence.get_task("queue_task_001").await.unwrap();
    assert!(deleted_task.is_none());
}
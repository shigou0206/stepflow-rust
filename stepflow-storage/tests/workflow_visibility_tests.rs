use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::{PersistenceManager, PersistenceManagerImpl};
use stepflow_sqlite::models::workflow_visibility::{WorkflowVisibility, UpdateWorkflowVisibility};
use chrono::Utc;

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_workflow_visibility_persistence_full() {
    // 初始化数据库
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    // 创建持久化管理器实例
    let persistence = PersistenceManagerImpl::new(pool.clone());

    let now = Utc::now().naive_utc();

    // 1. 测试创建 visibility
    let vis = WorkflowVisibility {
        run_id: "run_test_001".to_owned(),
        workflow_id: Some("wf_test_001".to_owned()),
        workflow_type: Some("type_test".to_owned()),
        start_time: Some(now),
        close_time: None,
        status: Some("RUNNING".to_owned()),
        memo: Some("initial visibility test".to_owned()),
        search_attrs: Some("key:test".to_owned()),
        version: 1,
    };

    persistence.create_visibility(&vis).await.unwrap();

    // 2. 测试根据 run_id 查询 visibility
    let fetched_vis = persistence.get_visibility("run_test_001").await.unwrap();
    assert!(fetched_vis.is_some());
    let fetched_vis = fetched_vis.unwrap();
    assert_eq!(fetched_vis.workflow_id.as_deref(), Some("wf_test_001"));
    assert_eq!(fetched_vis.status.as_deref(), Some("RUNNING"));

    // 3. 测试根据状态分页查询 visibility
    let visibilities = persistence.find_visibilities_by_status("RUNNING", 10, 0).await.unwrap();
    assert_eq!(visibilities.len(), 1);
    assert_eq!(visibilities[0].run_id, "run_test_001");

    // 4. 测试动态更新 visibility
    let update = UpdateWorkflowVisibility {
        status: Some("COMPLETED".to_owned()),
        close_time: Some(Utc::now().naive_utc()),
        memo: Some("visibility test completed".to_owned()),
        ..Default::default()
    };

    persistence.update_visibility("run_test_001", &update).await.unwrap();

    let updated_vis = persistence.get_visibility("run_test_001").await.unwrap().unwrap();
    assert_eq!(updated_vis.status.as_deref(), Some("COMPLETED"));
    assert_eq!(updated_vis.memo.as_deref(), Some("visibility test completed"));
    assert!(updated_vis.close_time.is_some());

    // 5. 测试删除 visibility
    persistence.delete_visibility("run_test_001").await.unwrap();

    let deleted_vis = persistence.get_visibility("run_test_001").await.unwrap();
    assert!(deleted_vis.is_none());
}
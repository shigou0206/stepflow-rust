use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::{PersistenceManager, PersistenceManagerImpl};
use stepflow_sqlite::models::workflow_state::{WorkflowState, UpdateWorkflowState};
use serde_json::json;
use chrono::Utc;

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_workflow_state_persistence_full() {
    // 初始化数据库
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    // 创建持久化管理器实例
    let persistence = PersistenceManagerImpl::new(pool.clone());

    // 1. 测试创建状态
    let state = WorkflowState {
        state_id: "state_001".to_owned(),
        run_id: "run_001".to_owned(),
        shard_id: 1,
        state_name: "initial".to_owned(),
        state_type: "Task".to_owned(),
        status: "running".to_owned(),
        input: Some(json!({"input_key": "input_value"}).to_string()),
        output: None,
        error: None,
        error_details: None,
        started_at: Some(Utc::now().naive_utc()),
        completed_at: None,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        version: 1,
    };

    persistence.create_state(&state).await.unwrap();

    // 2. 测试根据 state_id 查询状态
    let fetched_state = persistence.get_state("state_001").await.unwrap();
    assert!(fetched_state.is_some());
    assert_eq!(fetched_state.unwrap().state_name, "initial");

    // 3. 测试分页查询状态
    let states = persistence.find_states_by_run_id("run_001", 10, 0).await.unwrap();
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].state_id, "state_001");

    // 4. 测试更新状态
    let changes = UpdateWorkflowState {
        status: Some("completed".to_owned()),
        output: Some(json!({"output_key": "output_value"}).to_string()),
        completed_at: Some(Utc::now().naive_utc()),
        ..Default::default()
    };

    persistence.update_state("state_001", &changes).await.unwrap();

    let updated_state = persistence.get_state("state_001").await.unwrap().unwrap();
    assert_eq!(updated_state.status, "completed");
    assert!(updated_state.output.is_some());

    // 5. 测试删除状态
    persistence.delete_state("state_001").await.unwrap();
    let deleted_state = persistence.get_state("state_001").await.unwrap();
    assert!(deleted_state.is_none());
}
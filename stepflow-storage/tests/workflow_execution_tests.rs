use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::{PersistenceManager, PersistenceManagerImpl};
use stepflow_sqlite::models::workflow_execution::{WorkflowExecution, UpdateWorkflowExecution};
use serde_json::json;
use chrono::Utc;

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_workflow_execution_persistence_full() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    let persistence = PersistenceManagerImpl::new(pool.clone());

    let exec = WorkflowExecution {
        run_id: "run_001".to_owned(),
        workflow_id: Some("wf_001".to_owned()),
        shard_id: 1,
        template_id: Some("tpl_001".to_owned()),
        mode: "inline".to_owned(),
        current_state_name: Some("initial_state".to_owned()),
        status: "running".to_owned(),
        workflow_type: "type_1".to_owned(),
        input: Some(json!({"key": "value"}).to_string()),
        input_version: 0,
        result: None,
        result_version: 0,
        start_time: Utc::now().naive_utc()  ,
        close_time: None,
        current_event_id: 0,
        memo: None,
        search_attrs: None,
        context_snapshot: None,
        version: 0,
    };

    persistence.create_execution(&exec).await.unwrap();

    // 查询确认创建成功
    let fetched_exec = persistence.get_execution("run_001").await.unwrap();
    assert!(fetched_exec.is_some());

    // 分页查询
    let executions = persistence.find_executions(10, 0).await.unwrap();
    assert_eq!(executions.len(), 1);

    // 状态查询
    let executions_by_status = persistence.find_executions_by_status("running", 10, 0).await.unwrap();
    assert_eq!(executions_by_status.len(), 1);

    // 更新
    let changes = UpdateWorkflowExecution {
        status: Some("completed".to_owned()),
        current_state_name: Some("final_state".to_owned()),
        result: Some(json!({"result_key": "result_value"}).to_string()),
        close_time: Some(Utc::now().naive_utc()),
        ..Default::default()
    };
    persistence.update_execution("run_001", &changes).await.unwrap();

    // 删除
    persistence.delete_execution("run_001").await.unwrap();
    let deleted_exec = persistence.get_execution("run_001").await.unwrap();
    assert!(deleted_exec.is_none());
}
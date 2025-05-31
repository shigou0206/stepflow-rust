use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::{PersistenceManager, PersistenceManagerImpl};
use stepflow_sqlite::models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate};
use chrono::Utc;

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_workflow_template_persistence_full() {
    // 初始化数据库
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    // 创建持久化管理器实例
    let persistence = PersistenceManagerImpl::new(pool.clone());

    let now = Utc::now().naive_utc();

    // 1. 测试创建模板
    let tpl = WorkflowTemplate {
        template_id: "tpl_001".to_owned(),
        name: "Basic Workflow".to_owned(),
        description: Some("A basic workflow template".to_owned()),
        dsl_definition: r#"{"steps": []}"#.to_owned(),
        version: 1,
        created_at: now,
        updated_at: now,
    };

    persistence.create_template(&tpl).await.unwrap();

    // 2. 测试根据 template_id 查询模板
    let fetched_tpl = persistence.get_template("tpl_001").await.unwrap();
    assert!(fetched_tpl.is_some());
    assert_eq!(fetched_tpl.unwrap().name, "Basic Workflow");

    // 3. 测试分页查询模板
    let templates = persistence.find_templates(10, 0).await.unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].template_id, "tpl_001");

    // 4. 测试更新模板
    let changes = UpdateWorkflowTemplate {
        name: Some("Advanced Workflow".to_owned()),
        description: Some("An advanced workflow template".to_owned()),
        dsl_definition: None, // 测试部分更新
        version: Some(2),
    };

    persistence.update_template("tpl_001", &changes).await.unwrap();

    let updated_tpl = persistence.get_template("tpl_001").await.unwrap().unwrap();
    assert_eq!(updated_tpl.name, "Advanced Workflow");
    assert_eq!(updated_tpl.description.unwrap(), "An advanced workflow template");
    assert_eq!(updated_tpl.version, 2);
    assert_eq!(updated_tpl.dsl_definition, r#"{"steps": []}"#); // 确认未更新字段未被修改

    // 5. 测试删除模板
    persistence.delete_template("tpl_001").await.unwrap();
    let deleted_tpl = persistence.get_template("tpl_001").await.unwrap();
    assert!(deleted_tpl.is_none());
}
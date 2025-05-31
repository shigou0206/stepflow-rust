use sqlx::{SqlitePool, migrate::Migrator};
use stepflow_storage::{PersistenceManager, PersistenceManagerImpl};
use stepflow_sqlite::models::timer::{Timer, UpdateTimer};
use chrono::{Utc, Duration};

static MIGRATOR: Migrator = sqlx::migrate!("../stepflow-sqlite/migrations");

#[tokio::test]
async fn test_timer_persistence_full() {
    // 初始化数据库
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.expect("Migration failed");

    // 创建持久化管理器实例
    let persistence = PersistenceManagerImpl::new(pool.clone());

    let now = Utc::now().naive_utc();

    // 1. 测试创建 timer
    let timer = Timer {
        timer_id: "timer_001".to_owned(),
        run_id: "run_001".to_owned(),
        shard_id: 1,
        fire_at: now + Duration::seconds(60),
        status: "scheduled".to_owned(),
        version: 1,
        state_name: "initial_state".to_owned(),
    };

    persistence.create_timer(&timer).await.unwrap();

    // 2. 测试根据 timer_id 查询 timer
    let fetched_timer = persistence.get_timer("timer_001").await.unwrap();
    assert!(fetched_timer.is_some());
    assert_eq!(fetched_timer.unwrap().status, "scheduled");

    // 3. 测试更新 timer
    let changes = UpdateTimer {
        status: Some("fired".to_owned()),
        fire_at: Some(now + Duration::seconds(30)),
        version: Some(2),
        state_name: Some("updated_state".to_owned()),
    };

    persistence.update_timer("timer_001", &changes).await.unwrap();

    let updated_timer = persistence.get_timer("timer_001").await.unwrap().unwrap();
    assert_eq!(updated_timer.status, "fired");
    assert_eq!(updated_timer.version, 2);
    assert_eq!(updated_timer.state_name, "updated_state");

    // 4. 测试查询指定时间前的 timers
    let timers_due = persistence.find_timers_before(now + Duration::seconds(31), 10).await.unwrap();
    assert_eq!(timers_due.len(), 1);
    assert_eq!(timers_due[0].timer_id, "timer_001");

    // 5. 测试删除 timer
    persistence.delete_timer("timer_001").await.unwrap();
    let deleted_timer = persistence.get_timer("timer_001").await.unwrap();
    assert!(deleted_timer.is_none());
}
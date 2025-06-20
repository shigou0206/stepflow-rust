use crate::error::AppResult;
use crate::service::{TemplateService, ExecutionService, ActivityTaskService, WorkflowEventService, QueueTaskService, TimerService};
use stepflow_dto::dto::{template::*, execution::*, activity_task::*, workflow_event::*, queue_task::*, timer::*};
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct DummyServiceImpl;

macro_rules! impl_dummy_service {
    ($trait:path, $($fn:ident($($arg:tt)*): $ret:ty),* $(,)?) => {
        #[async_trait::async_trait]
        impl $trait for DummyServiceImpl {
            $(
                async fn $fn(&self, $($arg)*) -> $ret {
                    Err(crate::error::AppError::Internal(
                        concat!(stringify!($trait), "::", stringify!($fn), " not implemented").into()
                    ))
                }
            )*
        }
    };
}

impl_dummy_service!(TemplateService,
    create(_dto: TemplateUpsert): AppResult<TemplateDto>,
    update(_id: &str, _dto: TemplateUpsert): AppResult<TemplateDto>,    
    get(_id: &str): AppResult<TemplateDto>,
    list(): AppResult<Vec<TemplateDto>>,
    delete(_id: &str): AppResult<()>,
);

impl_dummy_service!(ExecutionService,
    start(_req: ExecStart): AppResult<ExecDto>,
    get(_run_id: &str): AppResult<ExecDto>,
    list(_limit: i64, _offset: i64): AppResult<Vec<ExecDto>>,
    update(_run_id: &str, _status: String, _result: Option<Value>): AppResult<()>,
    delete(_run_id: &str): AppResult<()>,
    list_by_status(_status: &str, _limit: i64, _offset: i64): AppResult<Vec<ExecDto>>,
);

impl_dummy_service!(ActivityTaskService,
    list_tasks(_limit: i64, _offset: i64): AppResult<Vec<ActivityTaskDto>>,
    get_task(_task_token: &str): AppResult<ActivityTaskDto>,
    get_tasks_by_run_id(_run_id: &str): AppResult<Vec<ActivityTaskDto>>,
    start_task(_task_token: &str): AppResult<ActivityTaskDto>,
    complete_task(_task_token: &str, _result: Value): AppResult<ActivityTaskDto>,
    fail_task(_task_token: &str, _req: FailRequest): AppResult<ActivityTaskDto>,
    heartbeat_task(_task_token: &str, _req: HeartbeatRequest): AppResult<ActivityTaskDto>,
);

impl_dummy_service!(WorkflowEventService,
    list_events(_limit: i64, _offset: i64): AppResult<Vec<WorkflowEventDto>>,
    get_event(_id: i64): AppResult<Option<WorkflowEventDto>>,
    list_events_for_run(_run_id: &str, _limit: i64, _offset: i64): AppResult<Vec<WorkflowEventDto>>,
    record_event(_req: RecordEventRequest): AppResult<WorkflowEventDto>,
    archive_event(_id: i64): AppResult<Option<WorkflowEventDto>>,
    delete_event(_id: i64): AppResult<()>,
);

impl_dummy_service!(QueueTaskService,
    get_task(_task_id: &str): AppResult<QueueTaskDto>,
    list_tasks_by_status(_status: &str, _limit: i64, _offset: i64): AppResult<Vec<QueueTaskDto>>,
    update_task(_task_id: &str, _update: UpdateQueueTaskDto): AppResult<()>,
    delete_task(_task_id: &str): AppResult<()>,
    list_tasks_to_retry(_before: chrono::NaiveDateTime, _limit: i64): AppResult<Vec<QueueTaskDto>>,
);

impl_dummy_service!(TimerService,
    create_timer(_dto: CreateTimerDto): AppResult<TimerDto>,
    get_timer(_timer_id: &str): AppResult<TimerDto>,
    update_timer(_timer_id: &str, _update: UpdateTimerDto): AppResult<TimerDto>,
    delete_timer(_timer_id: &str): AppResult<()>,
    find_timers_before(_before: DateTime<Utc>, _limit: i64): AppResult<Vec<TimerDto>>,
);
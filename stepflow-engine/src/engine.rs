//! WorkflowEngine – orchestration core (inline + deferred)
//
//! 调度顺序： Input-Mapping → handler → Output-Mapping → merge → 下一状态
//! handler 只关心业务数据，Mapping 统一收敛在 Engine。

use chrono::{DateTime, Utc};
use log::debug;
use serde_json::Value;
use sqlx::{Sqlite, Transaction, SqlitePool, Acquire};
use stepflow_dsl::{state::base::BaseState, State, WorkflowDSL};
use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use stepflow_storage::PersistenceManager;
use stepflow_sqlite::models::queue_task::{QueueTask, UpdateQueueTask};
use uuid::Uuid;

use crate::{
    command::{step_once, Command},
    handler,
    mapping::MappingPipeline,
};

// ────────────────────────────────────────────────────────────────────
// 公用枚举 / Traits
// ────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkflowMode {
    Inline,
    Deferred,
}

#[async_trait::async_trait]
pub trait TaskStore: Send + Sync {
    async fn insert_task(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<(), String>;

    async fn update_task_status(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        run_id: &str,
        state_name: &str,
        status: &str,
        result: &Value,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
pub trait TaskQueue: Send + Sync {
    async fn push(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        run_id: &str,
        state_name: &str,
    ) -> Result<(), String>;

    async fn pop(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<Option<(String, String)>, String>;
}

// ────────────────────────────────────────────────────────────────────
// 内存桩（单元测试 / inline）
// ────────────────────────────────────────────────────────────────────
#[cfg(feature = "memory_stub")]
pub mod memory_stub {
    use super::*;

    pub struct MemoryStore {
        persistence: Arc<dyn PersistenceManager>,
    }

    impl MemoryStore {
        pub fn new(persistence: Arc<dyn PersistenceManager>) -> Self {
            Self { persistence }
        }
    }

    #[async_trait::async_trait]
    impl TaskStore for MemoryStore {
        async fn insert_task(
            &self,
            _tx: &mut Transaction<'_, Sqlite>,
            run_id: &str,
            state_name: &str,
            resource: &str,
            input: &Value,
        ) -> Result<(), String> {
            let task = QueueTask {
                task_id: Uuid::new_v4().to_string(),
                run_id: run_id.to_string(),
                state_name: state_name.to_string(),
                task_payload: Some(input.to_string()),
                status: "pending".to_string(),
                attempts: 0,
                max_attempts: 3,
                error_message: None,
                last_error_at: None,
                next_retry_at: None,
                queued_at: Utc::now().naive_utc(),
                processing_at: None,
                completed_at: None,
                failed_at: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            };

            self.persistence.create_queue_task(&task)
                .await
                .map_err(|e| e.to_string())
        }

        async fn update_task_status(
            &self,
            _tx: &mut Transaction<'_, Sqlite>,
            run_id: &str,
            state_name: &str,
            status: &str,
            result: &Value,
        ) -> Result<(), String> {
            // 先查找对应的任务
            let tasks = self.persistence.find_queue_tasks_by_status("pending", 100, 0)
                .await
                .map_err(|e| e.to_string())?;
            
            if let Some(task) = tasks.into_iter().find(|t| t.run_id == run_id && t.state_name == state_name) {
                let now = Utc::now().naive_utc();
                let changes = match status {
                    "completed" => UpdateQueueTask {
                        status: Some(status.to_string()),
                        completed_at: Some(now),
                        updated_at: Some(now),
                        ..Default::default()
                    },
                    "failed" => {
                        let attempts = task.attempts + 1;
                        if attempts >= task.max_attempts {
                            UpdateQueueTask {
                                status: Some("failed".to_string()),
                                attempts: Some(attempts),
                                failed_at: Some(now),
                                error_message: Some(result.to_string()),
                                updated_at: Some(now),
                                ..Default::default()
                            }
                        } else {
                            // 计算下次重试时间，使用指数退避
                            let retry_delay = std::cmp::min(
                                30, // 最大30秒
                                5 * 2_i64.pow(attempts as u32) // 5s, 10s, 20s...
                            );
                            let next_retry = now + chrono::Duration::seconds(retry_delay);
                            
                            UpdateQueueTask {
                                status: Some("pending".to_string()),
                                attempts: Some(attempts),
                                error_message: Some(result.to_string()),
                                last_error_at: Some(now),
                                next_retry_at: Some(next_retry),
                                updated_at: Some(now),
                                ..Default::default()
                            }
                        }
                    },
                    _ => UpdateQueueTask {
                        status: Some(status.to_string()),
                        updated_at: Some(now),
                        ..Default::default()
                    }
                };

                self.persistence.update_queue_task(&task.task_id, &changes)
                    .await
                    .map_err(|e| e.to_string())
            } else {
                Err("Task not found".to_string())
            }
        }
    }

    pub struct MemoryQueue(pub Mutex<VecDeque<(String, String)>>);
    
    impl MemoryQueue {
        pub fn new() -> Self {
            Self(Mutex::new(VecDeque::new()))
        }
    }

    #[async_trait::async_trait]
    impl TaskQueue for MemoryQueue {
        async fn push(&self, _tx: &mut Transaction<'_, Sqlite>, run_id: &str, state_name: &str) -> Result<(), String> {
            self.0.lock().await.push_back((run_id.to_owned(), state_name.to_owned()));
            Ok(())
        }

        async fn pop(&self, _tx: &mut Transaction<'_, Sqlite>) -> Result<Option<(String, String)>, String> {
            Ok(self.0.lock().await.pop_front())
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 辅助类型
// ────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub should_continue: bool,
    pub updated_context: Value,
}

// ────────────────────────────────────────────────────────────────────
// Engine
// ────────────────────────────────────────────────────────────────────
pub struct WorkflowEngine<S: TaskStore, Q: TaskQueue> {
    pub run_id: String,
    pub dsl: WorkflowDSL,
    pub context: Value,
    pub current_state: String,

    pub mode: WorkflowMode,
    pub store: S,
    pub queue: Q,
    pub pool: SqlitePool,

    pub finished: bool,
    pub updated_at: DateTime<Utc>,
}

impl<S: TaskStore, Q: TaskQueue> WorkflowEngine<S, Q> {
    pub fn new(
        run_id: String,
        dsl: WorkflowDSL,
        input: Value,
        mode: WorkflowMode,
        store: S,
        queue: Q,
        pool: SqlitePool,
    ) -> Self {
        Self {
            run_id,
            current_state: dsl.start_at.clone(),
            dsl,
            context: input,
            mode,
            store,
            queue,
            pool,
            finished: false,
            updated_at: Utc::now(),
        }
    }

    // ─────────── Inline 一口气跑到底 ───────────
    pub async fn run_inline(mut self) -> Result<Value, String> {
        if self.mode != WorkflowMode::Inline {
            return Err("run_inline called on Deferred engine".into());
        }
        while !self.finished {
            self.advance_once().await?;
        }
        Ok(self.context)
    }

    // ─────────── 推进一步（Inline & Deferred 共用） ───────────
    pub async fn advance_once(&mut self) -> Result<StepOutcome, String> {
        if self.finished {
            return Ok(StepOutcome {
                should_continue: false,
                updated_context: self.context.clone(),
            });
        }

        // ① 译为 Command
        let cmd = step_once(&self.dsl, &self.current_state, &self.context)?;
        debug!(
            "[{}] step_once => {:?} @ {}",
            self.run_id,
            cmd.kind(),
            self.current_state
        );

        // ② 调度 & Mapping
        let (outcome, next_state_opt) = self.dispatch(cmd).await?;

        // ③ 更新 Engine 状态
        self.context = outcome.updated_context.clone();
        self.updated_at = Utc::now();

        if outcome.should_continue {
            self.current_state = next_state_opt.ok_or_else(|| {
                format!(
                    "State '{}' wants to continue but returned no next_state",
                    self.current_state
                )
            })?;
        } else {
            self.finished = true;
        }
        Ok(outcome)
    }

    // ─────────── 核心 dispatch（含双向 Mapping）───────────
    async fn dispatch(
        &mut self,
        cmd: Command,
    ) -> Result<(StepOutcome, Option<String>), String> {
        let mut conn = self.pool.acquire().await.map_err(|e| e.to_string())?;
        let mut tx = conn.begin().await.map_err(|e| e.to_string())?;
        let state_name = cmd.state_name();
        let state_enum = &self.dsl.states[&state_name];

        // ---- 提取 BaseState 引用 & 构建 MappingPipeline ----
        let base: &BaseState = match state_enum {
            State::Task(s) => &s.base,
            State::Wait(s) => &s.base,
            State::Pass(s) => &s.base,
            State::Choice(s) => &s.base,
            State::Fail(s) => &s.base,
            State::Succeed(s) => &s.base,
            State::Parallel(_) | State::Map(_) =>
                unimplemented!("Parallel / Map not yet supported"),
        };
        let pipeline = MappingPipeline {
            input_mapping: base.input_mapping.as_ref(),
            output_mapping: base.output_mapping.as_ref(),
        };

        // ---- ① Input-Mapping ----
        let exec_in = pipeline.apply_input(&self.context)?;

        // ---- ② 调 handler (返回 raw_out, logical_next) ----
        let (raw_out, logical_next): (Value, Option<String>) = match (cmd, state_enum) {
            (Command::ExecuteTask { .. }, State::Task(t)) => (
                handler::handle_task(
                    self.mode,
                    &self.run_id,
                    &state_name,
                    t,
                    &exec_in,
                    &self.store,
                    &self.queue,
                    &mut tx,
                )
                .await?,
                t.base.next.clone(),
            ),
            (Command::Wait { .. }, State::Wait(w)) => (
                handler::handle_wait(&state_name, w, &exec_in).await?,
                w.base.next.clone(),
            ),
            (Command::Pass { .. }, State::Pass(p)) => (
                handler::handle_pass(&state_name, p, &exec_in).await?,
                p.base.next.clone(),
            ),
            (Command::Choice { next_state, .. }, State::Choice(c)) => (
                handler::handle_choice(&state_name, c, &exec_in).await?,
                Some(next_state),
            ),
            (Command::Succeed { output, .. }, _) => {
                let out = handler::handle_succeed(&state_name, &output).await?;
                (out, None)
            }
            // ─────── Fail 状态直接报错 ───────
            (Command::Fail { error, cause, .. }, State::Fail(_f)) => {
                // 这里的 error/cause 都是 Option<String>，我们用 unwrap_or_default() 解包
                let err_msg = error.unwrap_or_default();
                let cause_msg = cause.unwrap_or_default();
                let full = if cause_msg.is_empty() {
                    err_msg.clone()
                } else {
                    format!("{}: {}", err_msg, cause_msg)
                };
                return Err(full);
            }
            _ => unreachable!("command/state mismatch"),
        };

        tx.commit().await.map_err(|e| e.to_string())?;
        
        // ---- ③ Output-Mapping + merge ----
        let new_ctx = pipeline.apply_output(&raw_out, &self.context)?;

        Ok((
            StepOutcome {
                should_continue: logical_next.is_some(),
                updated_context: new_ctx,
            },
            logical_next,
        ))
    }
}

// ─────────── Command 辅助方法 ───────────
impl Command {
    #[inline]
    fn state_name(&self) -> String {
        match self {
            Command::ExecuteTask { state_name, .. }
            | Command::Wait { state_name, .. }
            | Command::Pass { state_name, .. }
            | Command::Choice { state_name, .. }
            | Command::Succeed { state_name, .. }
            | Command::Fail { state_name, .. } => state_name.clone(),
        }
    }

    #[inline]
    fn kind(&self) -> &'static str {
        match self {
            Command::ExecuteTask { .. } => "Task",
            Command::Wait { .. } => "Wait",
            Command::Pass { .. } => "Pass",
            Command::Choice { .. } => "Choice",
            Command::Succeed { .. } => "Succeed",
            Command::Fail { .. } => "Fail",
        }
    }
}
//! WorkflowEngine – orchestration core (inline + deferred)
//
//! 调度顺序： Input-Mapping → handler → Output-Mapping → merge → 下一状态
//! handler 只关心业务数据，Mapping 统一收敛在 Engine。

use chrono::{DateTime, Utc};
use log::debug;
use serde_json::Value;
use stepflow_dsl::{state::base::BaseState, State, WorkflowDSL};

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
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<(), String>;

    async fn update_task_status(
        &self,
        run_id: &str,
        state_name: &str,
        status: &str,
        result: &Value,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
pub trait TaskQueue: Send + Sync {
    async fn push(&self, run_id: &str, state_name: &str) -> Result<(), String>;
}

// ────────────────────────────────────────────────────────────────────
// 内存桩（单元测试 / inline）
// ────────────────────────────────────────────────────────────────────
#[cfg(feature = "memory_stub")]
pub mod memory_stub {
    use super::*;
    use std::collections::VecDeque;
    use tokio::sync::Mutex;

    pub struct MemoryStore;
    #[async_trait::async_trait]
    impl TaskStore for MemoryStore {
        async fn insert_task(
            &self,
            _run: &str,
            _st: &str,
            _res: &str,
            _inp: &Value,
        ) -> Result<(), String> {
            Ok(())
        }
        async fn update_task_status(
            &self,
            _run: &str,
            _st: &str,
            _status: &str,
            _res: &Value,
        ) -> Result<(), String> {
            Ok(())
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
        async fn push(&self, run_id: &str, state_name: &str) -> Result<(), String> {
            self.0
                .lock()
                .await
                .push_back((run_id.to_owned(), state_name.to_owned()));
            Ok(())
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
    ) -> Self {
        Self {
            run_id,
            current_state: dsl.start_at.clone(),
            dsl,
            context: input,
            mode,
            store,
            queue,
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
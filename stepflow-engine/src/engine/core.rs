use crate::command::step_once;
use crate::handler::registry::StateHandlerRegistry;
use crate::signal::handler::apply_signal;
use chrono::{DateTime, Utc};
use log::{debug, warn};
use serde_json::Value;
use std::sync::Arc;
use stepflow_dsl::{State, WorkflowDSL};
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::signal::ExecutionSignal;
use stepflow_hook::EngineEventDispatcher;
use stepflow_storage::db::DynPM;
use stepflow_storage::entities::workflow_execution::UpdateStoredWorkflowExecution;
use tokio::sync::mpsc;

use super::{
    dispatch::dispatch_command,
    types::{StateExecutionResult, StepOutcome},
};

/// 状态执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateStatus {
    Started,
    Completed,
    Failed,
    Cancelled,
}

impl StateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            StateStatus::Started => "STARTED",
            StateStatus::Completed => "COMPLETED",
            StateStatus::Failed => "FAILED",
            StateStatus::Cancelled => "CANCELLED",
        }
    }
}

pub type SignalSender = mpsc::UnboundedSender<ExecutionSignal>;
pub type SignalReceiver = mpsc::UnboundedReceiver<ExecutionSignal>;

pub struct WorkflowEngine {
    pub run_id: String,
    pub dsl: WorkflowDSL,
    pub context: Value,
    pub current_state: String,
    pub last_task_state: Option<String>, // 记录上一个 Task 状态

    pub parent_run_id: Option<String>,
    pub parent_state_name: Option<String>,

    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub persistence: DynPM,
    pub state_handler_registry: Arc<StateHandlerRegistry>,

    pub finished: bool,
    pub updated_at: DateTime<Utc>,

    // Signal handling
    signal_sender: Option<SignalSender>,
    signal_receiver: Option<SignalReceiver>,
}

impl WorkflowEngine {
    pub fn new(
        run_id: String,
        dsl: WorkflowDSL,
        input: Value,
        event_dispatcher: Arc<EngineEventDispatcher>,
        persistence: DynPM,
        state_handler_registry: Arc<StateHandlerRegistry>,
    ) -> Self {
        let (signal_sender, signal_receiver) = mpsc::unbounded_channel();

        Self {
            run_id,
            current_state: dsl.start_at.clone(),
            last_task_state: None, // 初始化为 None
            parent_run_id: None,
            parent_state_name: None,
            dsl,
            context: input,
            event_dispatcher,
            persistence,
            state_handler_registry,
            finished: false,
            updated_at: Utc::now(),
            signal_sender: Some(signal_sender),
            signal_receiver: Some(signal_receiver),
        }
    }

    // Signal handling methods
    pub fn get_signal_sender(&self) -> Option<SignalSender> {
        self.signal_sender.clone()
    }

    pub fn set_signal_channel(&mut self, sender: SignalSender, receiver: SignalReceiver) {
        self.signal_sender = Some(sender);
        self.signal_receiver = Some(receiver);
    }

    // pub async fn handle_next_signal(&mut self) -> Result<bool, String> {
    //     // 先把 signal_receiver 移出来，避免后续借用冲突
    //     let mut receiver = match self.signal_receiver.take() {
    //         Some(rx) => rx,
    //         None => return Ok(false),
    //     };

    //     let mut handled_any = false;

    //     loop {
    //         match receiver.try_recv() {
    //             Ok(signal) => {
    //                 apply_signal(self, signal).await?;
    //                 handled_any = true;
    //             }
    //             Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
    //                 self.signal_receiver = Some(receiver);
    //                 break;
    //             }
    //             Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
    //                 warn!("Signal channel disconnected for run_id: {}", self.run_id);
    //                 break;
    //             }
    //         }
    //     }

    //     Ok(handled_any)
    // }

    pub async fn handle_next_signal(&mut self) -> Result<Option<StateExecutionResult>, String> {
        let mut receiver = match self.signal_receiver.take() {
            Some(rx) => rx,
            None => return Ok(None),
        };

        while let Ok(signal) = receiver.try_recv() {
            let result = apply_signal(self, signal).await?;
            self.signal_receiver = Some(receiver);
            return Ok(Some(result)); // ✅ 只处理一个 signal，返回结果
        }

        self.signal_receiver = Some(receiver);
        Ok(None)
    }

    pub(crate) async fn dispatch_event(&self, ev: EngineEvent) {
        self.event_dispatcher.dispatch(ev).await;
    }
    pub(crate) fn state_def(&self) -> &State {
        &self.dsl.states[&self.current_state]
    }

    pub async fn advance_until_blocked(&mut self) -> Result<StateExecutionResult, String> {
        loop {
            if self.finished {
                return Ok(StateExecutionResult {
                    output: self.context.clone(),
                    next_state: Some(self.current_state.clone()),
                    should_continue: false,
                    metadata: None,
                    is_blocking: false,
                });
            }

            debug!("🔁 advance_once old_state={}", self.current_state);
            let step_out = self.advance_once().await?;
            debug!(
                "🔁 advance_once done | should_continue={} | is_blocking={} | new_state={}",
                step_out.should_continue, step_out.is_blocking, self.current_state
            );

            if !step_out.should_continue || step_out.is_blocking {
                debug!(
                    "⏸ suspend or end: should_continue={}, is_blocking={}",
                    step_out.should_continue, step_out.is_blocking
                );
                return Ok(StateExecutionResult {
                    output: self.context.clone(),
                    next_state: Some(self.current_state.clone()),
                    should_continue: false,
                    metadata: None,
                    is_blocking: step_out.is_blocking,
                });
            }

            // ✳️ 如果不挂起，处理完 signal 再继续
            if let Err(e) = self.handle_next_signal().await {
                return Err(e);
            }
        }
    }

    // ---- 只在首次进入节点时写 input ---------------------------------
    async fn record_state_started(&self) -> Result<(), String> {
        use stepflow_storage::entities::workflow_state::UpdateStoredWorkflowState;

        let state_id = format!("{}:{}", self.run_id, self.current_state);
        let state_type = match self.state_def() {
            State::Task(_) => "Task",
            State::Choice(_) => "Choice",
            State::Pass(_) => "Pass",
            State::Wait(_) => "Wait",
            State::Fail(_) => "Fail",
            State::Succeed(_) => "Succeed",
            State::Parallel(_) => "Parallel",
            State::Map(_) => "Map",
        };

        self.persistence
            .update_state(
                &state_id,
                &UpdateStoredWorkflowState {
                    state_name: Some(self.current_state.clone()),
                    state_type: Some(state_type.into()),
                    status: Some("STARTED".into()),
                    input: Some(Some(self.context.clone())), // ✅ 只在这里写入
                    started_at: Some(Some(Utc::now().naive_utc())),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| e.to_string())
    }

    // ---- 完成 / 失败时，只更新 output & status ------------------------
    pub async fn record_state_finished(
        &self,
        success: bool,
        output: Option<Value>,
        error: Option<String>,
    ) -> Result<(), String> {
        use stepflow_storage::entities::workflow_state::UpdateStoredWorkflowState;

        let state_id = format!("{}:{}", self.run_id, self.current_state);

        self.persistence
            .update_state(
                &state_id,
                &UpdateStoredWorkflowState {
                    status: Some(if success { "COMPLETED" } else { "FAILED" }.into()),
                    output: Some(output),
                    error: Some(error),
                    completed_at: Some(Some(Utc::now().naive_utc())),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| e.to_string())
    }

    // ------------------ 单步执行 -------------------------------
    async fn advance_once(&mut self) -> Result<StepOutcome, String> {
        if self.finished {
            return Ok(StepOutcome {
                should_continue: false,
                updated_context: self.context.clone(),
                is_blocking: false,
            });
        }

        // ① NodeEnter
        self.dispatch_event(EngineEvent::NodeEnter {
            run_id: self.run_id.clone(),
            state_name: self.current_state.clone(),
            input: self.context.clone(),
        })
        .await;

        // ② 记录 STARTED
        self.record_state_started().await?;

        // ③ step_once & dispatch handler
        let cmd = step_once(&self.dsl, &self.current_state, &self.context)?;
        debug!(
            "[{}] step_once => {:?} @ {}",
            self.run_id,
            cmd.kind(),
            self.current_state
        );

        let (outcome, next_state_opt, _raw_out, _meta) = dispatch_command(
            &cmd,
            self.state_def(),
            &self.context,
            &self.run_id,
            &self.persistence,
            &self.state_handler_registry,
        )
        .await?;

        // ④ 更新 context
        self.context = outcome.updated_context.clone();
        self.updated_at = Utc::now();

        // ⑤ 写 COMPLETED 状态
        self.record_state_finished(outcome.should_continue, Some(self.context.clone()), None)
            .await?;

        // ⑥ NodeExit
        self.dispatch_event(EngineEvent::NodeExit {
            run_id: self.run_id.clone(),
            state_name: self.current_state.clone(),
            status: if outcome.should_continue {
                "success"
            } else {
                "failed"
            }
            .into(),
            duration_ms: Some((Utc::now() - self.updated_at).num_milliseconds() as u64),
        })
        .await;

        // ⑦ 构造 execution 更新
        let mut exec_update = UpdateStoredWorkflowExecution {
            context_snapshot: Some(Some(self.context.clone())),
            ..Default::default()
        };

        // ⑧ 正常推进
        if outcome.should_continue {
            let next = next_state_opt.ok_or_else(|| {
                format!(
                    "state {} should continue but next_state is None",
                    self.current_state
                )
            })?;

            if matches!(self.state_def(), State::Task(_)) {
                self.last_task_state = Some(self.current_state.clone());
            }

            self.current_state = next.clone();
            exec_update.current_state_name = Some(Some(next));
        }

        // ⑨ 非挂起 + 不再继续 ⇒ 要么终结（end），要么报错
        if !outcome.should_continue && !outcome.is_blocking {
            if self.dsl.is_end_state(&self.current_state) {
                // ✅ 确实是终结节点，正常 finalize
                self.finalize().await?;
                exec_update.status = Some("COMPLETED".into());
                exec_update.close_time = Some(Some(self.updated_at.naive_utc()));
            } else {
                // ❌ 非 end 却尝试终结 ⇒ DSL 错误
                return Err(format!(
                    "State '{}' returned should_continue = false but is not an end state",
                    self.current_state
                ));
            }
        }

        // ⑩ 保存 execution 状态
        self.persistence
            .update_execution(&self.run_id, &exec_update)
            .await
            .map_err(|e| e.to_string())?;

        debug!("🔁 step_once returned: {:?}", cmd);
        debug!("📤 step outcome: {:?}", outcome);

        Ok(outcome)
    }

    pub async fn finalize(&mut self) -> Result<(), String> {
        if self.finished {
            return Ok(()); // 避免重复 finalize
        }

        self.finished = true;

        let update = UpdateStoredWorkflowExecution {
            status: Some("COMPLETED".into()),
            close_time: Some(Some(Utc::now().naive_utc())),
            ..Default::default()
        };

        self.persistence
            .update_execution(&self.run_id, &update)
            .await
            .map_err(|e| e.to_string())?;

        self.dispatch_event(EngineEvent::WorkflowFinished {
            run_id: self.run_id.clone(),
            result: self.context.clone(),
        })
        .await;

        if let (Some(parent_run_id), Some(state_name)) =
            (self.parent_run_id.clone(), self.parent_state_name.clone())
        {
            self.dispatch_event(EngineEvent::SubflowFinished {
                parent_run_id,
                child_run_id: self.run_id.clone(),
                state_name,
                result: self.context.clone(),
            })
            .await;
        }

        Ok(())
    }
    // ------------------------- 恢复 ----------------------------------

    pub async fn restore(
        run_id: String,
        event_dispatcher: Arc<EngineEventDispatcher>,
        persistence: DynPM,
        state_handler_registry: Arc<StateHandlerRegistry>,
    ) -> Result<Self, String> {
        let execution = persistence
            .get_execution(&run_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", run_id))?;

        // ✅ 从 dsl_definition 字段恢复 DSL
        let dsl: WorkflowDSL = match execution.dsl_definition.clone() {
            Some(Value::Object(_)) => serde_json::from_value(execution.dsl_definition.unwrap())
                .map_err(|e| format!("Invalid DSL JSON object: {}", e))?,
            Some(Value::String(s)) => {
                serde_json::from_str(&s).map_err(|e| format!("Invalid DSL string: {}", e))?
            }
            Some(_) => return Err("Invalid DSL format".to_string()),
            None => return Err("Missing DSL definition".to_string()),
        };

        let context = execution
            .context_snapshot
            .unwrap_or_else(|| Value::Object(Default::default()));

        let current_state = execution
            .current_state_name
            .unwrap_or_else(|| dsl.start_at.clone());

        let finished = matches!(
            execution.status.as_str(),
            "COMPLETED" | "FAILED" | "TERMINATED" | "PAUSED" | "SUSPENDED"
        );

        let (signal_sender, signal_receiver) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            run_id,
            current_state,
            last_task_state: None,
            parent_run_id: execution.parent_run_id,
            parent_state_name: execution.parent_state_name,
            dsl,
            context,
            event_dispatcher,
            persistence,
            state_handler_registry,
            finished,
            updated_at: execution
                .close_time
                .unwrap_or_else(|| execution.start_time)
                .and_utc(),
            signal_sender: Some(signal_sender),
            signal_receiver: Some(signal_receiver),
        })
    }
}

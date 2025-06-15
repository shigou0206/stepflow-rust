use crate::command::step_once;
use crate::mapping::MappingPipeline;
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
use crate::handler::registry::StateHandlerRegistry;
use tokio::sync::mpsc;

use super::{
    dispatch::dispatch_command,
    types::{StateExecutionResult, StepOutcome, WorkflowMode},
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

    pub mode: WorkflowMode,
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
        mode: WorkflowMode,
        event_dispatcher: Arc<EngineEventDispatcher>,
        persistence: DynPM,
        state_handler_registry: Arc<StateHandlerRegistry>,
    ) -> Self {
        let (signal_sender, signal_receiver) = mpsc::unbounded_channel();

        Self {
            run_id,
            current_state: dsl.start_at.clone(),
            last_task_state: None, // 初始化为 None
            dsl,
            context: input,
            mode,
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

    pub async fn handle_next_signal(&mut self) -> Result<bool, String> {
        let receiver = match &mut self.signal_receiver {
            Some(rx) => rx,
            None => return Ok(false),
        };

        match receiver.try_recv() {
            Ok(signal) => {
                apply_signal(self, signal).await?;
                Ok(true)
            }
            Err(mpsc::error::TryRecvError::Empty) => Ok(false),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                warn!("Signal channel disconnected for run_id: {}", self.run_id);
                self.signal_receiver = None;
                Ok(false)
            }
        }
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

        let template_id = execution
            .template_id
            .ok_or_else(|| "Template ID missing".to_string())?;

        let template = persistence
            .get_template(&template_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Template {} not found", template_id))?;

        let dsl: WorkflowDSL =
            serde_json::from_str(&template.dsl_definition).map_err(|e| e.to_string())?;

        let context = execution
            .context_snapshot
            .unwrap_or_else(|| Value::Object(Default::default()));

        let current_state = execution
            .current_state_name
            .unwrap_or_else(|| dsl.start_at.clone());

        let mode = match execution.mode.as_str() {
            "INLINE" => WorkflowMode::Inline,
            "DEFERRED" => WorkflowMode::Deferred,
            _ => return Err(format!("Invalid mode {}", execution.mode)),
        };

        let finished = matches!(
            execution.status.as_str(),
            "COMPLETED" | "FAILED" | "TERMINATED" | "PAUSED" | "SUSPENDED"
        );

        let (signal_sender, signal_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            run_id,
            current_state,
            last_task_state: None,
            dsl,
            context,
            mode,
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

    // --------------------- 调度辅助 -----------------------------

    pub(crate) async fn dispatch_event(&self, ev: EngineEvent) {
        self.event_dispatcher.dispatch(ev).await;
    }
    pub(crate) fn state_def(&self) -> &State {
        &self.dsl.states[&self.current_state]
    }
    fn deferred_task(&self) -> bool {
        self.mode == WorkflowMode::Deferred && matches!(self.state_def(), State::Task(_))
    }

    // --------------------- 主入口 -------------------------------

    pub async fn run_inline(&mut self) -> Result<Value, String> {
        self.run_state().await
    }

    pub async fn advance_until_blocked(&mut self) -> Result<StateExecutionResult, String> {
        let out = self.run_state().await?;
        Ok(StateExecutionResult {
            output: out,
            next_state: Some(self.current_state.clone()),
            should_continue: !self.finished,
            metadata: None,
        })
    }

    async fn run_state(&mut self) -> Result<Value, String> {
        if self.mode == WorkflowMode::Inline {
            self.dispatch_event(EngineEvent::WorkflowStarted {
                run_id: self.run_id.clone(),
            })
            .await;
        }
        loop {
            if self.finished {
                break;
            }

            // 记录当前 state 是否是 Task（SendData 之类）
            let is_task_state = matches!(self.state_def(), State::Task(_));

            let step_out = self.advance_once().await?;
            debug!(
                "🔁 advance_once done | should_continue={} | new_state={}",
                step_out.should_continue, self.current_state
            );

            if !step_out.should_continue {
                break; // End 节点
            }

            // ---- 如果刚才执行的就是 Task 状态，说明任务已写入队列；立即挂起 ----
            if is_task_state && self.mode == WorkflowMode::Deferred {
                debug!("⏸ task scheduled, engine suspend");
                break; // 立即退出，等待外部 worker 通过 /update 触发信号处理
            }

            // ---- 处理 task 完成/失败的情况 ----
            if self.check_deferred().await? {
                break;
            }

            // Handle any pending signals at the end of each loop
            if let Err(e) = self.handle_next_signal().await {
                return Err(e);
            }
        }
        debug!(
            "🔚 loop exit | run_id={} | state={}",
            self.run_id, self.current_state
        );
        Ok(self.context.clone())
    }
    // ------------------ Deferred 轮询 ---------------------------

    async fn check_deferred(&mut self) -> Result<bool, String> {
        if !self.deferred_task() {
            return Ok(false);
        }

        let maybe_task = self
            .persistence
            .find_queue_tasks_by_status("pending", 100, 0)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|t| t.run_id == self.run_id && t.state_name == self.current_state);

        if let Some(t) = maybe_task {
            match t.status.as_str() {
                "completed" => {
                    if let Some(payload) = t.task_payload {
                        let State::Task(task_state) = self.state_def() else {
                            return Err("Expected Task state".into());
                        };
                        let pipeline = MappingPipeline {
                            input_mapping: task_state.base.input_mapping.as_ref(),
                            output_mapping: task_state.base.output_mapping.as_ref(),
                        };
                        self.context = pipeline
                            .apply_output(&payload, &self.context)
                            .map_err(|e| e.to_string())?;

                        self.persistence
                            .update_execution(
                                &self.run_id,
                                &UpdateStoredWorkflowExecution {
                                    context_snapshot: Some(Some(self.context.clone())),
                                    ..Default::default()
                                },
                            )
                            .await
                            .map_err(|e| e.to_string())?;

                        self.dispatch_event(EngineEvent::NodeSuccess {
                            run_id: self.run_id.clone(),
                            state_name: self.current_state.clone(),
                            output: payload,
                        })
                        .await;
                    }
                    Ok(true)
                }
                "failed" => {
                    let err_msg = t.error_message.unwrap_or_else(|| "Task failed".to_string());
                    self.persistence
                        .update_execution(
                            &self.run_id,
                            &UpdateStoredWorkflowExecution {
                                status: Some("FAILED".into()),
                                close_time: Some(Some(Utc::now().naive_utc())),
                                ..Default::default()
                            },
                        )
                        .await
                        .map_err(|e| e.to_string())?;
                    self.dispatch_event(EngineEvent::NodeFailed {
                        run_id: self.run_id.clone(),
                        state_name: self.current_state.clone(),
                        error: err_msg.clone(),
                    })
                    .await;
                    Err(err_msg)
                }
                _ => Ok(true),
            }
        } else {
            Ok(false)
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
    async fn record_state_finished(
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
                    // ⛔ 不触碰 input
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| e.to_string())
    }

    // ------------------ 单步执行 -------------------------------

    async fn advance_once(&mut self) -> Result<StepOutcome, String> {
        // 已结束就直接返回
        if self.finished {
            return Ok(StepOutcome {
                should_continue: false,
                updated_context: self.context.clone(),
            });
        }

        // —— ① NodeEnter & 记录 STARTED ——
        self.dispatch_event(EngineEvent::NodeEnter {
            run_id: self.run_id.clone(),
            state_name: self.current_state.clone(),
            input: self.context.clone(),
        })
        .await;

        // ⚠️ 只在首次进入时插入，避免后续覆盖 input
        self.record_state_started().await?;

        // —— ② 真正执行当前节点 ——
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
            self.mode,
            &self.persistence,
            &self.state_handler_registry,
        )
        .await?;

        // 更新本地 context
        self.context = outcome.updated_context.clone();
        self.updated_at = Utc::now();

        // —— ③ 记录 COMPLETED/FAILED，只更新 output/status ——
        self.record_state_finished(
            /* success */ outcome.should_continue,
            /* output  */ Some(self.context.clone()),
            /* error   */ None,
        )
        .await?;

        // 发送 NodeExit
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

        // —— ④ 推进游标 or 结束工作流 ——
        let mut exec_update = UpdateStoredWorkflowExecution {
            context_snapshot: Some(Some(self.context.clone())),
            ..Default::default()
        };

        if outcome.should_continue {
            let next = next_state_opt.ok_or_else(|| {
                format!(
                    "state {} should continue but next_state is None",
                    self.current_state
                )
            })?;

            // Task 节点：记下 “上一个 task”
            if matches!(self.state_def(), State::Task(_)) {
                self.last_task_state = Some(self.current_state.clone());
            }

            self.current_state = next.clone();
            exec_update.current_state_name = Some(Some(next));
        } else {
            self.finished = true;
            exec_update.status = Some("COMPLETED".into());
            exec_update.close_time = Some(Some(self.updated_at.naive_utc()));
            self.dispatch_event(EngineEvent::WorkflowFinished {
                run_id: self.run_id.clone(),
                result: self.context.clone(),
            })
            .await;
        }

        // 写 execution 表
        self.persistence
            .update_execution(&self.run_id, &exec_update)
            .await
            .map_err(|e| e.to_string())?;

        debug!("🔁 step_once returned: {:?}", cmd);
        debug!("📤 step outcome: {:?}", outcome);

        Ok(outcome)
    }

    // ----------------- 外部控制 -------------------------------

    pub async fn pause(&mut self) -> Result<(), String> {
        self.persistence
            .update_execution(
                &self.run_id,
                &UpdateStoredWorkflowExecution {
                    status: Some("PAUSED".into()),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| e.to_string())?;
        self.finished = true;
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<(), String> {
        let exec = self
            .persistence
            .get_execution(&self.run_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", self.run_id))?;

        match exec.status.as_str() {
            "PAUSED" | "SUSPENDED" => {
                self.finished = false;
                Ok(())
            }
            s => Err(format!("Cannot resume from status {}", s)),
        }
    }
}

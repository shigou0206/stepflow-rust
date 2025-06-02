use chrono::{DateTime, Utc};
use log::debug;
use serde_json::Value;
use sqlx::{SqlitePool, Acquire};
use stepflow_dsl::{State, WorkflowDSL};
use std::sync::Arc;
use stepflow_storage::PersistenceManager;
use stepflow_hook::{EngineEvent, EngineEventDispatcher};

use crate::command::step_once;

use super::{
    types::{WorkflowMode, StepOutcome, StateExecutionResult},
    traits::{TaskStore, TaskQueue},
    dispatch::dispatch_command,
};

pub struct WorkflowEngine<S: TaskStore, Q: TaskQueue> {
    pub run_id: String,
    pub dsl: WorkflowDSL,
    pub context: Value,
    pub current_state: String,

    pub mode: WorkflowMode,
    pub store: S,
    pub queue: Q,
    pub pool: SqlitePool,
    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub persistence: Arc<dyn PersistenceManager>,

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
        event_dispatcher: Arc<EngineEventDispatcher>,
        persistence: Arc<dyn PersistenceManager>,
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
            event_dispatcher,
            persistence,
            finished: false,
            updated_at: Utc::now(),
        }
    }

    /// 从数据库恢复工作流引擎状态
    pub async fn restore(
        run_id: String,
        dsl: WorkflowDSL,
        store: S,
        queue: Q,
        pool: SqlitePool,
        event_dispatcher: Arc<EngineEventDispatcher>,
        persistence: Arc<dyn PersistenceManager>,
    ) -> Result<Self, String> {
        // 1. 从 DB 加载执行记录
        let execution = persistence
            .get_execution(&run_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", run_id))?;

        // 2. 解析上下文数据
        let context = if let Some(ctx_str) = execution.context_snapshot {
            serde_json::from_str(&ctx_str)
                .map_err(|e| format!("Failed to parse context: {}", e))?
        } else {
            Value::Object(Default::default())
        };

        // 3. 确定当前状态
        let current_state = execution.current_state_name
            .unwrap_or_else(|| dsl.start_at.clone());

        // 4. 解析执行模式
        let mode = match execution.mode.as_str() {
            "INLINE" => WorkflowMode::Inline,
            "DEFERRED" => WorkflowMode::Deferred,
            _ => return Err(format!("Invalid mode: {}", execution.mode)),
        };

        // 5. 确定是否已完成
        let finished = matches!(execution.status.as_str(), "COMPLETED" | "FAILED");

        Ok(Self {
            run_id,
            current_state,
            dsl,
            context,
            mode,
            store,
            queue,
            pool,
            event_dispatcher,
            persistence,
            finished,
            updated_at: execution.close_time
                .unwrap_or_else(|| execution.start_time)
                .and_utc(),
        })
    }

    pub async fn run_inline(&mut self) -> Result<Value, String> {
        let result = self.run_state().await?;
        Ok(result)
    }

    pub async fn advance_until_blocked(&mut self) -> Result<StateExecutionResult, String> {
        let result = self.run_state().await?;
        Ok(StateExecutionResult {
            output: result,
            next_state: Some(self.current_state.clone()),
            should_continue: !self.finished,
        })
    }

    async fn run_state(&mut self) -> Result<Value, String> {
        if self.mode == WorkflowMode::Inline {
            self.event_dispatcher.dispatch(EngineEvent::WorkflowStarted { 
                run_id: self.run_id.clone() 
            }).await;
        }

        loop {
            if self.finished {
                debug!("[Engine] Workflow finished, breaking loop");
                break;
            }

            // Check deferred task status first
            if self.check_deferred_task_status().await? {
                break;
            }

            let step_result = self.advance_once().await?;

            if !step_result.should_continue || self.is_deferred_task() {
                debug!("[Engine] Stopping at state: {}", self.current_state);
                break;
            }
        }

        Ok(self.context.clone())
    }

    async fn check_deferred_task_status(&mut self) -> Result<bool, String> {
        if !self.is_deferred_task() {
            return Ok(false);
        }

        let mut conn = self.pool.acquire().await.map_err(|e| e.to_string())?;
        let tx = conn.begin().await.map_err(|e| e.to_string())?;
        
        let tasks = self.persistence.as_ref()
            .find_queue_tasks_by_status("pending", 100, 0)
            .await
            .map_err(|e| e.to_string())?;
        
        if let Some(task) = tasks.into_iter()
            .find(|t| t.run_id == self.run_id && t.state_name == self.current_state) 
        {
            match task.status.as_str() {
                "completed" => {
                    if let Some(payload) = task.task_payload {
                        let result: Value = serde_json::from_str(&payload)
                            .map_err(|e| format!("Failed to parse task result: {}", e))?;
                        
                        self.context = result.clone();
                        
                        self.dispatch_event(EngineEvent::NodeSuccess {
                            run_id: self.run_id.clone(),
                            state_name: self.current_state.clone(),
                            output: result,
                        }).await;
                    }
                    tx.commit().await.map_err(|e| e.to_string())?;
                    Ok(true)
                }
                "failed" => {
                    let error = task.error_message.unwrap_or_else(|| "Task failed".to_string());
                    
                    self.dispatch_event(EngineEvent::NodeFailed {
                        run_id: self.run_id.clone(),
                        state_name: self.current_state.clone(),
                        error: error.clone(),
                    }).await;
                    
                    tx.commit().await.map_err(|e| e.to_string())?;
                    Err(error)
                }
                _ => {
                    debug!("[Engine] Task {} is still in progress (status: {})", 
                           task.task_id, task.status);
                    tx.commit().await.map_err(|e| e.to_string())?;
                    Ok(true)
                }
            }
        } else {
            tx.commit().await.map_err(|e| e.to_string())?;
            Ok(false)
        }
    }

    pub async fn advance_once(&mut self) -> Result<StepOutcome, String> {
        if self.finished {
            return Ok(StepOutcome {
                should_continue: false,
                updated_context: self.context.clone(),
            });
        }

        self.dispatch_event(EngineEvent::NodeEnter {
            run_id: self.run_id.clone(),
            state_name: self.current_state.clone(),
            input: self.context.clone(),
        }).await;

        let cmd = step_once(&self.dsl, &self.current_state, &self.context)?;
        debug!(
            "[{}] step_once => {:?} @ {}",
            self.run_id,
            cmd.kind(),
            self.current_state
        );

        let mut conn = self.pool.acquire().await.map_err(|e| e.to_string())?;
        let mut tx = conn.begin().await.map_err(|e| e.to_string())?;

        let (outcome, next_state_opt) = dispatch_command(
            &cmd,
            self.current_state_type(),
            &self.context,
            &self.run_id,
            self.mode,
            &self.store,
            &self.queue,
            &mut tx,
            self.event_dispatcher.clone(),
            self.persistence.clone(),
        ).await?;

        tx.commit().await.map_err(|e| e.to_string())?;

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
            self.dispatch_event(EngineEvent::WorkflowFinished {
                run_id: self.run_id.clone(),
                result: self.context.clone(),
            }).await;
        }

        Ok(outcome)
    }

    // 辅助方法：事件分发
    async fn dispatch_event(&self, event: EngineEvent) {
        self.event_dispatcher.dispatch(event).await;
    }

    // 辅助方法：获取当前状态类型
    fn current_state_type(&self) -> &State {
        &self.dsl.states[&self.current_state]
    }

    // 辅助方法：检查是否是延迟任务
    fn is_deferred_task(&self) -> bool {
        self.mode == WorkflowMode::Deferred && 
        matches!(self.current_state_type(), State::Task(_))
    }
} 
use chrono::{DateTime, Utc};
use log::{debug, warn};
use serde_json::Value;
use stepflow_dsl::{State, WorkflowDSL};
use std::sync::Arc;
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_hook::{EngineEvent, EngineEventDispatcher};
use stepflow_storage::entities::workflow_execution::UpdateStoredWorkflowExecution;
use stepflow_match::service::MatchService;
use stepflow_match::queue::{TaskStore, TaskQueue};
use crate::mapping::MappingPipeline;


use crate::command::step_once;

use super::{
    types::{WorkflowMode, StepOutcome, StateExecutionResult},
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
    pub event_dispatcher: Arc<EngineEventDispatcher>,
    pub persistence: Arc<dyn PersistenceManager>,
    pub match_service: Arc<dyn MatchService>,

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
        event_dispatcher: Arc<EngineEventDispatcher>,
        persistence: Arc<dyn PersistenceManager>,
        match_service: Arc<dyn MatchService>,
    ) -> Self {
        Self {
            run_id,
            current_state: dsl.start_at.clone(),
            dsl,
            context: input,
            mode,
            store,
            queue,
            event_dispatcher,
            persistence,
            match_service,
            finished: false,
            updated_at: Utc::now(),
        }
    }

    /// 从数据库恢复工作流引擎状态
    pub async fn restore(
        run_id: String,
        store: S,
        queue: Q,
        event_dispatcher: Arc<EngineEventDispatcher>,
        persistence: Arc<dyn PersistenceManager>,
        match_service: Arc<dyn MatchService>,
    ) -> Result<Self, String> {
        // 1. 从 DB 加载执行记录
        let execution = persistence
            .get_execution(&run_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", run_id))?;

        // 2. 从模板加载 DSL
        let template_id = execution.template_id
            .ok_or_else(|| "Template ID not found in execution record".to_string())?;
            
        let template = persistence
            .get_template(&template_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Template {} not found", template_id))?;

        let dsl: WorkflowDSL = serde_json::from_str(&template.dsl_definition)
            .map_err(|e| format!("Failed to parse DSL from template: {}", e))?;

        // 3. 解析上下文数据
        let context = if let Some(ctx) = execution.context_snapshot {
            ctx
        } else {
            warn!(
                "[Engine] Missing context_snapshot for workflow {} (status: {}, mode: {}), using empty object as default",
                run_id,
                execution.status,
                execution.mode
            );
            Value::Object(Default::default())
        };

        // 4. 确定当前状态
        let current_state = execution.current_state_name
            .unwrap_or_else(|| dsl.start_at.clone());

        // 5. 解析执行模式
        let mode = match execution.mode.as_str() {
            "INLINE" => WorkflowMode::Inline,
            "DEFERRED" => WorkflowMode::Deferred,
            _ => return Err(format!("Invalid mode: {}", execution.mode)),
        };

        // 6. 确定工作流状态
        let finished = match execution.status.as_str() {
            // 终态
            "COMPLETED" | "FAILED" | "TERMINATED" => true,
            // 中间态
            "RUNNING" | "PENDING" => false,
            // 暂停态 - 需要手动恢复
            "PAUSED" | "SUSPENDED" => true,
            // 其他状态视为异常
            status => return Err(format!("Invalid workflow status: {}", status)),
        };

        Ok(Self {
            run_id,
            current_state,
            dsl,
            context,
            mode,
            store,
            queue,
            event_dispatcher,
            persistence,
            match_service,
            finished,
            updated_at: execution.close_time
                .unwrap_or_else(|| execution.start_time)
                .and_utc(),
        })
    }

    /// 恢复暂停的工作流
    pub async fn resume(&mut self) -> Result<(), String> {
        // 检查当前状态
        let execution = self.persistence
            .get_execution(&self.run_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", self.run_id))?;

        match execution.status.as_str() {
            "PAUSED" | "SUSPENDED" => {
                self.finished = false;
                Ok(())
            }
            status => Err(format!("Cannot resume workflow in {} status", status)),
        }
    }

    /// 暂停工作流
    pub async fn pause(&mut self) -> Result<(), String> {
        // 更新数据库状态
        let update = UpdateStoredWorkflowExecution {
            status: Some("PAUSED".to_string()),
            ..Default::default()
        };
        
        self.persistence.update_execution(&self.run_id, &update)
            .await
            .map_err(|e| e.to_string())?;

        self.finished = true;
        Ok(())
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

        // 开始事务
        self.persistence.begin_transaction()
            .await
            .map_err(|e| e.to_string())?;
        
        let result = match self.persistence.as_ref()
            .find_queue_tasks_by_status("pending", 100, 0)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|t| t.run_id == self.run_id && t.state_name == self.current_state)
        {
            Some(task) => {
                match task.status.as_str() {
                    "completed" => {
                        if let Some(payload) = task.task_payload {
                            // 获取当前状态的 output mapping
                            let state = self.current_state_type();
                            let base = match state {
                                State::Task(s) => &s.base,
                                _ => return Err("Expected Task state".to_string()),
                            };

                            let pipeline = MappingPipeline {
                                input_mapping: base.input_mapping.as_ref(),
                                output_mapping: base.output_mapping.as_ref(),
                            };

                            // 应用 output mapping
                            let new_ctx = pipeline
                                .apply_output(&payload, &self.context)
                                .map_err(|e| e.to_string())?;
                            
                            self.context = new_ctx;
                            
                            // 更新执行记录
                            let update = UpdateStoredWorkflowExecution {
                                context_snapshot: Some(Some(self.context.clone())),
                                ..Default::default()
                            };
                            
                            self.persistence.update_execution(&self.run_id, &update)
                                .await
                                .map_err(|e| e.to_string())?;
                            
                            self.dispatch_event(EngineEvent::NodeSuccess {
                                run_id: self.run_id.clone(),
                                state_name: self.current_state.clone(),
                                output: payload,
                            }).await;
                        }
                        Ok(true)
                    }
                    "failed" => {
                        let error = task.error_message.unwrap_or_else(|| "Task failed".to_string());
                        
                        // 更新执行记录
                        let update = UpdateStoredWorkflowExecution {
                            status: Some("FAILED".to_string()),
                            close_time: Some(Some(Utc::now().naive_utc())),
                            ..Default::default()
                        };
                        
                        self.persistence.update_execution(&self.run_id, &update)
                            .await
                            .map_err(|e| e.to_string())?;
                        
                        self.dispatch_event(EngineEvent::NodeFailed {
                            run_id: self.run_id.clone(),
                            state_name: self.current_state.clone(),
                            error: error.clone(),
                        }).await;
                        
                        Err(error)
                    }
                    _ => {
                        debug!("[Engine] Task {} is still in progress (status: {})", 
                               task.task_id, task.status);
                        Ok(true)
                    }
                }
            }
            None => Ok(false)
        };

        match result {
            Ok(r) => {
                // 提交事务
                self.persistence.commit()
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(r)
            }
            Err(e) => {
                // 回滚事务
                if let Err(rollback_err) = self.persistence.rollback().await {
                    warn!("Failed to rollback transaction: {}", rollback_err);
                }
                Err(e)
            }
        }
    }

    async fn advance_once(&mut self) -> Result<StepOutcome, String> {
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

        // 开始事务
        self.persistence.begin_transaction()
            .await
            .map_err(|e| e.to_string())?;

        let result = match dispatch_command(
            &cmd,
            self.current_state_type(),
            &self.context,
            &self.run_id,
            self.mode,
            &self.store,
            self.match_service.clone(),
            &self.persistence,
            self.event_dispatcher.clone(),
            self.persistence.clone(),
        ).await {
            Ok((outcome, next_state_opt)) => {
                // 更新上下文和时间戳
                self.context = outcome.updated_context.clone();
                self.updated_at = Utc::now();

                // 准备数据库更新
                let mut update = UpdateStoredWorkflowExecution {
                    context_snapshot: Some(Some(self.context.clone())),
                    ..Default::default()
                };

                if outcome.should_continue {
                    let next_state = next_state_opt.ok_or_else(|| {
                        format!(
                            "State '{}' wants to continue but returned no next_state",
                            self.current_state
                        )
                    })?;
                    
                    // 更新当前状态
                    self.current_state = next_state.clone();
                    update.current_state_name = Some(Some(next_state));
                } else {
                    self.finished = true;
                    update.status = Some("COMPLETED".to_string());
                    update.close_time = Some(Some(self.updated_at.naive_utc()));
                    
                    self.dispatch_event(EngineEvent::WorkflowFinished {
                        run_id: self.run_id.clone(),
                        result: self.context.clone(),
                    }).await;
                }

                // 更新执行记录
                self.persistence.update_execution(&self.run_id, &update)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(outcome)
            }
            Err(e) => {
                // 回滚事务
                if let Err(rollback_err) = self.persistence.rollback().await {
                    warn!("Failed to rollback transaction: {}", rollback_err);
                }
                Err(e)
            }
        }?;

        // 提交事务
        self.persistence.commit()
            .await
            .map_err(|e| e.to_string())?;

        Ok(result)
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
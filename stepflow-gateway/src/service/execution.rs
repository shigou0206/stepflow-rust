
use anyhow::{Context, Error};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use stepflow_core::{
    app_state::AppState,
    error::{AppError, AppResult},
};
use stepflow_dto::dto::execution::*;
use stepflow_engine::engine::{WorkflowEngine, WorkflowMode};
use stepflow_storage::entities::workflow_execution::StoredWorkflowExecution;
use stepflow_storage::error::StorageError;

#[derive(Clone, Debug)]
pub struct ExecutionSqlxSvc {
    state: Arc<AppState>,
}

impl ExecutionSqlxSvc {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    fn mode_from_str(s: &str) -> Result<WorkflowMode, AppError> {
        match s {
            "INLINE" => Ok(WorkflowMode::Inline),
            "DEFERRED" => Ok(WorkflowMode::Deferred),
            _ => Err(AppError::BadRequest(
                "mode must be INLINE or DEFERRED".into(),
            )),
        }
    }
}

#[async_trait]
impl crate::service::ExecutionService for ExecutionSqlxSvc {
    async fn start(&self, req: ExecStart) -> AppResult<ExecDto> {
        // ① 准备 DSL
        let dsl_val = if let Some(tpl_id) = &req.template_id {
            let tpl = self.state.persist.get_template(tpl_id).await
                .map_err(|e: StorageError| Error::new(e))?
                .ok_or(AppError::NotFound)?;
            serde_json::from_str::<Value>(&tpl.dsl_definition)
                .context("解析模板 DSL 失败")?
        } else {
            req.dsl.clone()
                .ok_or(AppError::BadRequest("dsl or template_id required".into()))?
        };

        let dsl = match dsl_val {
            Value::Object(_) => {
                serde_json::from_value(dsl_val)
                    .map_err(|e| AppError::BadRequest(format!("invalid DSL: {e}")))?
            }
            Value::String(ref s) => {
                serde_json::from_str(s)
                    .map_err(|e| AppError::BadRequest(format!("invalid DSL string: {e}")))?
            }
            _ => {
                return Err(AppError::BadRequest("DSL must be a JSON object or JSON string".into()));
            }
        };

        // ---------- ② 生成 run_id / 创建引擎 ----------
        let mode = Self::mode_from_str(&req.mode)?;
        let run_id = uuid::Uuid::new_v4().to_string();

        let mut engine = WorkflowEngine::new(
            run_id.clone(),
            dsl,
            req.init_ctx.clone().unwrap_or_default(),
            mode,
            self.state.event_dispatcher.clone(),
            self.state.persist.clone(),
            self.state.state_handler_registry.clone(),
        );

        // ---------- ③ 先落库 execution(状态 RUNNING) 防止锁冲突 ----------
        let started_at = chrono::Utc::now();
        let exec_row = StoredWorkflowExecution {
            run_id: run_id.clone(),
            workflow_id: Some(format!("wf-{run_id}")),
            shard_id: 0,
            template_id: req.template_id.clone(),
            mode: req.mode.clone(),
            current_state_name: Some("initial".into()),
            status: "RUNNING".into(),
            workflow_type: "default".into(),
            input: Some(req.init_ctx.clone().unwrap_or_default()),
            input_version: 1,
            result: None,
            result_version: 1,
            start_time: started_at.naive_utc(),
            close_time: None,
            current_event_id: 0,
            memo: None,
            search_attrs: None,
            context_snapshot: None,
            version: 1,
        };
        self.state
            .persist
            .create_execution(&exec_row)
            .await
            .map_err(|e: StorageError| AppError::Internal(e.to_string()))?;

        // ---------- ④ 执行引擎 ----------
        let (final_status, final_result, finished_at) = match mode {
            WorkflowMode::Inline => {
                let res = engine
                    .run_inline()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("执行工作流失败: {e}")))?;
                ("COMPLETED".to_string(), Some(res), Some(chrono::Utc::now()))
            }
            WorkflowMode::Deferred => {
                engine.advance_until_blocked().await.ok(); // 允许 None
                self.state
                    .engines
                    .lock()
                    .await
                    .insert(run_id.clone(), engine);
                ("RUNNING".to_string(), None, None)
            }
        };

        // ---------- ⑤ 若已结束则补丁更新 execution ----------
        if final_status != "RUNNING" {
            use stepflow_storage::entities::workflow_execution::UpdateStoredWorkflowExecution;
            let patch = UpdateStoredWorkflowExecution {
                status: Some(final_status.clone()),
                result: Some(final_result.clone()),
                close_time: finished_at.map(|t| Some(t.naive_utc())),
                result_version: Some(2),
                ..Default::default()
            };
            self.state
                .persist
                .update_execution(&run_id, &patch)
                .await
                .map_err(|e: StorageError| AppError::Internal(e.to_string()))?;
        }

        // ---------- ⑥ 返回 DTO ----------
        Ok(ExecDto {
            run_id,
            mode: req.mode,
            status: final_status,
            result: final_result,
            started_at,
            finished_at,
        })
    }
    async fn get(&self, id: &str) -> AppResult<ExecDto> {
        let row = self
            .state
            .persist
            .get_execution(id)
            .await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(ExecDto {
            run_id: row.run_id,
            mode: row.mode,
            status: row.status,
            result: row.result,
            started_at: row.start_time.and_utc(),
            finished_at: Option::map(row.close_time, |t| t.and_utc()),
        })
    }

    async fn list(&self, limit: i64, offset: i64) -> AppResult<Vec<ExecDto>> {
        let rows = self
            .state
            .persist
            .find_executions(limit, offset)
            .await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(rows
            .into_iter()
            .map(|r| ExecDto {
                run_id: r.run_id,
                mode: r.mode,
                status: r.status,
                result: r.result,
                started_at: r.start_time.and_utc(),
                finished_at: Option::map(r.close_time, |t| t.and_utc()),
            })
            .collect::<Vec<_>>())
    }

    async fn update(&self, run_id: &str, status: String, result: Option<Value>) -> AppResult<()> {
        let update =
            stepflow_storage::entities::workflow_execution::UpdateStoredWorkflowExecution {
                status: Some(status),
                result: Some(result),
                result_version: Some(2),
                ..Default::default()
            };
        self.state
            .persist
            .update_execution(run_id, &update)
            .await
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("update execution failed: {}", e)))?;
        Ok(())
    }

    async fn delete(&self, run_id: &str) -> AppResult<()> {
        self.state
            .persist
            .delete_execution(run_id)
            .await
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("delete execution failed: {}", e)))?;
        Ok(())
    }

    async fn list_by_status(
        &self,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<ExecDto>> {
        let rows = self
            .state
            .persist
            .find_executions_by_status(status, limit, offset)
            .await
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("list_by_status failed: {}", e)))?;
        Ok(rows
            .into_iter()
            .map(|r| ExecDto {
                run_id: r.run_id,
                mode: r.mode,
                status: r.status,
                result: r.result,
                started_at: r.start_time.and_utc(),
                finished_at: Option::map(r.close_time, |t| t.and_utc()),
            })
            .collect())
    }
}

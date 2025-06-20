// ✅ ExecutionSqlxSvc：已补全对 parent_run_id / parent_state_name / dsl_definition 字段支持
use anyhow::{Context, Error};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use crate::{
    app_state::AppState,
    error::{AppError, AppResult},
};
use stepflow_dto::dto::execution::*;
use stepflow_engine::engine::WorkflowEngine;
use stepflow_storage::entities::workflow_execution::{StoredWorkflowExecution, UpdateStoredWorkflowExecution};
use stepflow_storage::error::StorageError;
use stepflow_dsl::dsl::WorkflowDSL;

#[derive(Clone, Debug)]
pub struct ExecutionSqlxSvc {
    state: Arc<AppState>,
}

impl ExecutionSqlxSvc {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl crate::service::ExecutionService for ExecutionSqlxSvc {

    async fn start(&self, req: ExecStart) -> AppResult<ExecDto> {
        // 1. 解析 DSL
        let dsl_val = if let Some(tpl_id) = &req.template_id {
            let tpl = self.state.persist.get_template(tpl_id).await
                .map_err(Error::new)?
                .ok_or(AppError::NotFound)?;
            serde_json::from_str::<Value>(&tpl.dsl_definition)
                .context("解析模板 DSL 失败")?
        } else {
            req.dsl.clone().ok_or_else(|| AppError::BadRequest("dsl or template_id required".into()))?
        };

        let dsl: WorkflowDSL = match dsl_val {
            Value::Object(_) => serde_json::from_value(dsl_val)
                .map_err(|e| AppError::BadRequest(format!("invalid DSL: {e}")))?,
            Value::String(ref s) => serde_json::from_str(s)
                .map_err(|e| AppError::BadRequest(format!("invalid DSL string: {e}")))?,
            _ => return Err(AppError::BadRequest("DSL must be a JSON object or JSON string".into())),
        };

        // 3. 确定 run_id
        let run_id = req.run_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let started_at = chrono::Utc::now();

        // 4. 构造引擎
        let mut engine = WorkflowEngine::new(
            run_id.clone(),
            dsl.clone(),
            req.init_ctx.clone().unwrap_or_default(),
            self.state.event_dispatcher.clone(),
            self.state.persist.clone(),
            self.state.state_handler_registry.clone(),
        );

        // 5. 插入初始 execution
        let exec_row = StoredWorkflowExecution {
            run_id: run_id.clone(),
            workflow_id: Some(format!("wf-{run_id}")),
            shard_id: 0,
            template_id: req.template_id.clone(),
            mode: "DEFERRED".into(),
            current_state_name: Some(dsl.start_at.clone()),
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
            parent_run_id: req.parent_run_id.clone(),
            parent_state_name: req.parent_state_name.clone(),
            dsl_definition: Some(serde_json::to_value(&dsl).map_err(|e| AppError::Internal(e.to_string()))?),
        };

        self.state.persist.create_execution(&exec_row)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 6. 执行一轮直到阻塞
        let result = engine.advance_until_blocked().await.ok();
        self.state.engines.lock().await.insert(run_id.clone(), engine);

        // 7. 补充更新 execution（仅上下文）
        let patch = UpdateStoredWorkflowExecution {
            context_snapshot: result.map(|r| Some(r.output)),
            ..Default::default()
        };

        self.state.persist.update_execution(&run_id, &patch).await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 8. 返回执行信息
        Ok(ExecDto {
            run_id,
            mode: "DEFERRED".into(),
            status: "RUNNING".into(),
            result: None,
            started_at,
            finished_at: None,
        })
    }

    async fn get(&self, id: &str) -> AppResult<ExecDto> {
        let row = self.state.persist.get_execution(id).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(ExecDto {
            run_id: row.run_id,
            mode: row.mode,
            status: row.status,
            result: row.result,
            started_at: row.start_time.and_utc(),
            finished_at: row.close_time.map(|t| t.and_utc()),
        })
    }

    async fn list(&self, limit: i64, offset: i64) -> AppResult<Vec<ExecDto>> {
        let rows = self.state.persist.find_executions(limit, offset).await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(rows.into_iter().map(|r| ExecDto {
            run_id: r.run_id,
            mode: r.mode,
            status: r.status,
            result: r.result,
            started_at: r.start_time.and_utc(),
            finished_at: r.close_time.map(|t| t.and_utc()),
        }).collect())
    }

    async fn update(&self, run_id: &str, status: String, result: Option<Value>) -> AppResult<()> {
        let update = stepflow_storage::entities::workflow_execution::UpdateStoredWorkflowExecution {
            status: Some(status),
            result: Some(result),
            result_version: Some(2),
            ..Default::default()
        };
        self.state.persist.update_execution(run_id, &update).await
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("update execution failed: {}", e)))?;
        Ok(())
    }

    async fn delete(&self, run_id: &str) -> AppResult<()> {
        self.state.persist.delete_execution(run_id).await
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("delete execution failed: {}", e)))?;
        Ok(())
    }

    async fn list_by_status(&self, status: &str, limit: i64, offset: i64) -> AppResult<Vec<ExecDto>> {
        let rows = self.state.persist.find_executions_by_status(status, limit, offset).await
            .map_err(|e: StorageError| AppError::Anyhow(anyhow::anyhow!("list_by_status failed: {}", e)))?;
        Ok(rows.into_iter().map(|r| ExecDto {
            run_id: r.run_id,
            mode: r.mode,
            status: r.status,
            result: r.result,
            started_at: r.start_time.and_utc(),
            finished_at: r.close_time.map(|t| t.and_utc()),
        }).collect())
    }
}

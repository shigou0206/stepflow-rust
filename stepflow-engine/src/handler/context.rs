//! engine/types.rs  —— WorkflowEngine 通用类型与运行时上下文

use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;

use stepflow_hook::{EngineEvent, EngineEventDispatcher};
use stepflow_match::queue::DynPM;
use stepflow_storage::entities::workflow_state::{
    StoredWorkflowState, UpdateStoredWorkflowState,
};

use crate::engine::WorkflowMode;

/// ------------------------------------------------------------
/// StateExecutionResult —— handler 的统一输出
/// ------------------------------------------------------------
#[derive(Debug)]
pub struct StateExecutionResult {
    pub output:          Value,
    pub next_state:      Option<String>,
    pub should_continue: bool,
}

/// ------------------------------------------------------------
/// StateExecutionContext —— handler 运行时上下文
/// ------------------------------------------------------------
pub struct StateExecutionContext<'a> {
    pub run_id:      &'a str,
    pub state_name:  &'a str,
    pub state_type:  &'a str,
    pub mode:        WorkflowMode,
    pub dispatcher:  &'a Arc<EngineEventDispatcher>,
    pub persistence: &'a DynPM,                // ✅ 改成别名
}

impl<'a> StateExecutionContext<'a> {
    pub fn new(
        run_id:      &'a str,
        state_name:  &'a str,
        state_type:  &'a str,
        mode:        WorkflowMode,
        dispatcher:  &'a Arc<EngineEventDispatcher>,
        persistence: &'a DynPM,                 // ✅ 改成别名
    ) -> Self {
        Self {
            run_id,
            state_name,
            state_type,
            mode,
            dispatcher,
            persistence,
        }
    }

    /* --------------------------------------------------------
     * DB helper methods
     * ----------------------------------------------------- */

    /// 新建一条 workflow_states 记录
    pub async fn create_state_record(&self, input: &Value) -> Result<(), String> {
        let now = Utc::now().naive_utc();

        let state = StoredWorkflowState {
            state_id:      format!("{}:{}", self.run_id, self.state_name),
            run_id:        self.run_id.to_owned(),
            shard_id:      1,                // ⚠️ 如有分片需求请自行调整
            state_name:    self.state_name.to_owned(),
            state_type:    self.state_type.to_owned(),
            status:        "started".into(),
            input:         Some(input.clone()),
            output:        None,
            error:         None,
            error_details: None,
            started_at:    Some(now),
            completed_at:  None,
            created_at:    now,
            updated_at:    now,
            version:       1,
        };

        self.persistence
            .create_state(&state)
            .await
            .map_err(|e| e.to_string())
    }

    /// 成功结束当前状态
    pub async fn update_success_state(&self, output: &Value) -> Result<(), String> {
        let update = UpdateStoredWorkflowState {
            status:       Some("succeeded".into()),
            output:       Some(Some(output.clone())),
            completed_at: Some(Some(Utc::now().naive_utc())),
            ..Default::default()
        };

        self.persistence
            .update_state(&format!("{}:{}", self.run_id, self.state_name), &update)
            .await
            .map_err(|e| e.to_string())
    }

    /// 失败结束当前状态
    pub async fn update_failure_state(&self, error_msg: &str) -> Result<(), String> {
        let update = UpdateStoredWorkflowState {
            status:       Some("failed".into()),
            error:        Some(Some(error_msg.into())),
            completed_at: Some(Some(Utc::now().naive_utc())),
            ..Default::default()
        };

        self.persistence
            .update_state(&format!("{}:{}", self.run_id, self.state_name), &update)
            .await
            .map_err(|e| e.to_string())
    }

    /* --------------------------------------------------------
     * Hook helpers
     * ----------------------------------------------------- */

    pub async fn dispatch_enter(&self, input: &Value) {
        self.dispatcher
            .dispatch(EngineEvent::NodeEnter {
                run_id:     self.run_id.into(),
                state_name: self.state_name.into(),
                input:      input.clone(),
            })
            .await;
    }

    pub async fn dispatch_success(&self, output: &Value) {
        self.dispatcher
            .dispatch(EngineEvent::NodeSuccess {
                run_id:     self.run_id.into(),
                state_name: self.state_name.into(),
                output:     output.clone(),
            })
            .await;
    }

    pub async fn dispatch_failure(&self, error_msg: &str) {
        self.dispatcher
            .dispatch(EngineEvent::NodeFailed {
                run_id:     self.run_id.into(),
                state_name: self.state_name.into(),
                error:      error_msg.into(),
            })
            .await;
    }
}
use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, warn};

use stepflow_dsl::{
    logic::ChoiceRule,
    state::{choice::ChoiceState, State},
};

use crate::logic::choice_eval::eval_choice_logic;
use crate::mapping::MappingPipeline;

use super::{StateExecutionScope, StateExecutionResult, StateHandler};

/// ---------------------------------------------------------------------
/// ChoiceHandler（无状态，可注册为单例）
/// ---------------------------------------------------------------------
pub struct ChoiceHandler;

impl ChoiceHandler {
    pub fn new() -> Self {
        Self
    }

    /// 真正的分支匹配逻辑
    async fn evaluate_choice(
        &self,
        state: &ChoiceState,
        input: &Value,
    ) -> Result<Option<String>, String> {
        debug!(
            "Evaluating choice rules with {} branches",
            state.choices.len()
        );

        for (idx, ChoiceRule { condition, next }) in state.choices.iter().enumerate() {
            match eval_choice_logic(condition, input) {
                Ok(true) => {
                    debug!("✅ Branch {idx} matched → {next}");
                    return Ok(Some(next.clone()));
                }
                Ok(false) => {
                    debug!("❌ Branch {idx} did not match");
                }
                Err(e) => {
                    warn!("⚠️ Failed to evaluate branch {idx}: {e}");
                    return Err(e);
                }
            }
        }

        if let Some(default_next) = &state.default_next {
            debug!("➕ No branches matched – using default → {default_next}");
            return Ok(Some(default_next.clone()));
        }

        warn!("❗ No matching branch and no default");
        Err("No matching branch and no default branch provided".into())
    }
}

#[async_trait]
impl StateHandler for ChoiceHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Choice(ref s) => s,
            _ => return Err("Invalid state type for ChoiceHandler".into()),
        };

        // ✅ input_mapping 支持
        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: None, // Choice 无需 output_mapping
        };

        let exec_input = pipeline.apply_input(&scope.context)?;

        let next_state = self.evaluate_choice(state, &exec_input).await?;

        Ok(StateExecutionResult {
            output: exec_input,            // ✅ 返回处理后的上下文
            next_state,                    // ✅ 下一状态（可能是 default）
            should_continue: true,        // ✅ 总是推进
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "choice"
    }

    async fn on_subflow_finished(
        &self,
        _scope: &StateExecutionScope<'_>,
        _parent_context: &Value,
        _child_run_id: &str,
        _result: &Value,
    ) -> Result<StateExecutionResult, String> {
        Err("on_subflow_finished not supported by this state".into())
    }
}
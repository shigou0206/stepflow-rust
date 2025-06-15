use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, warn};

use stepflow_dsl::{
    logic::ChoiceRule,
    state::{choice::ChoiceState, State},
};

use crate::logic::choice_eval::eval_choice_logic;
use super::{
    StateExecutionScope, StateExecutionResult, StateHandler,
};

/// ---------------------------------------------------------------------
/// ChoiceHandler（无状态，可注册为单例）
/// ---------------------------------------------------------------------
pub struct ChoiceHandler;

impl ChoiceHandler {
    pub fn new() -> Self {
        Self
    }

    /// 真正的分支匹配逻辑
    async fn evaluate_choice(&self, state: &ChoiceState, input: &Value) -> Result<Option<String>, String> {
        debug!(
            "Evaluating choice rules with {} branches",
            state.choices.len()
        );

        for (idx, ChoiceRule { condition, next }) in state.choices.iter().enumerate() {
            match eval_choice_logic(condition, input) {
                Ok(true) => {
                    debug!("Branch {idx} matched");
                    return Ok(Some(next.clone()));
                }
                Ok(false) => {
                    debug!("Branch {idx} did not match");
                }
                Err(e) => {
                    warn!("Failed to evaluate branch {idx}: {e}");
                    return Err(e);
                }
            }
        }

        if let Some(default_next) = &state.default_next {
            debug!("No branches matched – using default");
            return Ok(Some(default_next.clone()));
        }

        warn!("No matching branch and no default");
        Err("No matching branch and no default branch provided".into())
    }
}

/// ---------------------------------------------------------------------
/// 注册式 StateHandler 实现
/// ---------------------------------------------------------------------
#[async_trait]
impl StateHandler for ChoiceHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Choice(ref s) => s,
            _ => return Err("Invalid state type for ChoiceHandler".into()),
        };

        let next_state = self.evaluate_choice(state, input).await?;

        Ok(StateExecutionResult {
            output: input.clone(), // Choice 不修改上下文
            next_state,
            should_continue: true,
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "choice"
    }
}
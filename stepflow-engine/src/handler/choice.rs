use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, warn};

use stepflow_dsl::{
    logic::ChoiceRule,
    state::choice::ChoiceState,
};
  
use crate::logic::choice_eval::eval_choice_logic;
use super::{
    StateExecutionScope, StateExecutionResult, StateHandler,
};

/// ---------------------------------------------------------------------
/// ChoiceHandler
/// ---------------------------------------------------------------------
pub struct ChoiceHandler<'a> {
    state: &'a ChoiceState,
}

impl<'a> ChoiceHandler<'a> {
    pub fn new(state: &'a ChoiceState) -> Self {
        Self { state }
    }

    /// 真正的分支匹配逻辑
    async fn evaluate_choice(&self, input: &Value) -> Result<Option<String>, String> {
        debug!(
            "Evaluating choice rules with {} branches",
            self.state.choices.len()
        );

        // 1️⃣ 依次判断 DSL 中显式列出的 branches
        for (idx, ChoiceRule { condition, next }) in self.state.choices.iter().enumerate() {
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

        // 2️⃣ 无命中但存在 default
        if let Some(default_next) = &self.state.default_next {
            debug!("No branches matched – using default");
            return Ok(Some(default_next.clone()));
        }

        // 3️⃣ 无命中且无 default
        warn!("No matching branch and no default");
        Err("No matching branch and no default branch provided".into())
    }
}

/// ---------------------------------------------------------------------
/// StateHandler 实现
/// ---------------------------------------------------------------------
#[async_trait]
impl<'a> StateHandler for ChoiceHandler<'a> {
    async fn handle(
        &self,
        _scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let next_state = self.evaluate_choice(input).await?;

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
//! Choice handler – 只负责判断分支是否命中，
//! 并返回【未经修改】的上下文 JSON。

use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, warn};
use stepflow_dsl::{
    logic::ChoiceRule,
    state::choice::ChoiceState,
};
use std::sync::Arc;
use stepflow_storage::PersistenceManager;
use stepflow_hook::EngineEventDispatcher;

use crate::logic::choice_eval::eval_choice_logic;
use super::{StateHandler, StateExecutionContext, StateExecutionResult};

pub struct ChoiceHandler<'a> {
    state: &'a ChoiceState,
}

impl<'a> ChoiceHandler<'a> {
    pub fn new(state: &'a ChoiceState) -> Self {
        Self { state }
    }

    async fn evaluate_choice(&self, input: &Value) -> Result<Option<String>, String> {
        debug!("Evaluating choice rules with {} branches", self.state.choices.len());

        // 1. 遍历所有显式写在 DSL 里的 choice 规则
        for (index, ChoiceRule { condition, next }) in self.state.choices.iter().enumerate() {
            match eval_choice_logic(condition, input) {
                Ok(true) => {
                    debug!("Branch {} matched", index);
                    return Ok(Some(next.clone()));
                }
                Ok(false) => {
                    debug!("Branch {} did not match", index);
                    continue;
                }
                Err(e) => {
                    warn!("Failed to evaluate branch {}: {}", index, e);
                    return Err(e);
                }
            }
        }

        // 2. 如果没有任何 choice 命中，但有 default
        if let Some(default) = &self.state.default {
            debug!("No branches matched, using default");
            return Ok(Some(default.clone()));
        }

        // 3. 都没命中也没有 default
        warn!("No matching branch and no default");
        Err("No matching branch and no default branch provided".to_string())
    }
}

#[async_trait(?Send)]
impl<'a> StateHandler for ChoiceHandler<'a> {
    async fn handle(
        &self,
        _ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let next_state = self.evaluate_choice(input).await?;

        Ok(StateExecutionResult {
            output: input.clone(),
            next_state,
            should_continue: true,
        })
    }

    fn state_type(&self) -> &'static str {
        "choice"
    }
}

// 为了保持向后兼容，保留原有的函数签名
pub async fn handle_choice(
    state_name: &str,
    state: &ChoiceState,
    input: &Value,
    run_id: &str,
    event_dispatcher: &Arc<EngineEventDispatcher>,
    persistence: &Arc<dyn PersistenceManager>,
) -> Result<Value, String> {
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        crate::engine::WorkflowMode::Inline, // Choice 不区分模式
        event_dispatcher,
        persistence,
    );

    let handler = ChoiceHandler::new(state);
    let result = handler.execute(&ctx, input).await?;
    
    Ok(result.output)
}
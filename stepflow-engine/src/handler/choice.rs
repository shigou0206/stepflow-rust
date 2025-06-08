//! Choice handler – 只负责判断分支是否命中，
//! 并返回【未经修改】的上下文 JSON。

use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, warn};

use std::sync::Arc;

use stepflow_dsl::{
    logic::ChoiceRule,
    state::choice::ChoiceState,
};
use stepflow_hook::EngineEventDispatcher;
use stepflow_match::queue::DynPM;           // ✅ 统一别名
use crate::engine::WorkflowMode;       // Inline/Deferred 枚举

use crate::logic::choice_eval::eval_choice_logic;
use super::{
    StateExecutionContext, StateExecutionResult,
    StateHandler,
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
        if let Some(default) = &self.state.default {
            debug!("No branches matched – using default");
            return Ok(Some(default.clone()));
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
        _ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let next_state = self.evaluate_choice(input).await?;

        Ok(StateExecutionResult {
            output: input.clone(), // Choice 不修改上下文
            next_state,
            should_continue: true,
        })
    }

    fn state_type(&self) -> &'static str {
        "choice"
    }
}

/// ---------------------------------------------------------------------
/// 兼容旧调用方式的薄包装函数
/// ---------------------------------------------------------------------
pub async fn handle_choice(
    state_name: &str,
    state: &ChoiceState,
    input: &Value,
    run_id: &str,
    event_dispatcher: &Arc<EngineEventDispatcher>,
    persistence: &DynPM,                   // ✅ 统一使用 DynPM
) -> Result<Value, String> {
    // 构造执行上下文
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        "choice",
        WorkflowMode::Inline,              // Choice 无需区分模式
        event_dispatcher,
        persistence,
    );

    // 调用通用执行流程
    let handler = ChoiceHandler::new(state);
    let result = handler.execute(&ctx, input).await?;

    Ok(result.output)
}
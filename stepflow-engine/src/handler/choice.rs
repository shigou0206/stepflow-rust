//! Choice handler – 只负责判断分支是否命中，
//! 并返回【未经修改】的上下文 JSON。

use serde_json::Value;
use stepflow_dsl::{
    logic::ChoiceRule,
    state::choice::ChoiceState,
};

use crate::logic::choice_eval::eval_choice_logic;

/// Evaluate `ChoiceState` 并返回原上下文。
///
/// - 若任一 `choices` 条件满足，直接 `Ok(ctx.clone())`
/// - 若无命中但存在 `default` 分支，同样返回 `ctx.clone()`
/// - 否则报错
pub async fn handle_choice(
    state_name: &str,
    state: &ChoiceState,
    ctx: &Value,
) -> Result<Value, String> {
    // 1. 先遍历所有显式写在 DSL 里的 choice 规则
    for ChoiceRule { condition, .. } in &state.choices {
        if eval_choice_logic(condition, ctx)? {
            // 命中其中一个分支，直接返回上下文，不作修改
            return Ok(ctx.clone());
        }
    }

    // 2. 如果没有任何 choice 命中，但 DSL 写了 default
    if state.default.is_some() {
        return Ok(ctx.clone());
    }

    // 3. 都没命中也没有 default，则报错
    Err(format!("No matching branch in choice '{}'.", state_name))
}
//! Choice handler – 只负责判断分支是否命中，
//! 并返回【未经修改】的上下文 JSON。
//!
//! * “是否继续、跳到哪一分支” 已在 `engine::dispatch` 利用
//!   `Command::Choice { next_state, .. }` 处理，
//!   这里无需再关心。

use serde_json::Value;
use stepflow_dsl::{
    logic::ChoiceRule,
    state::choice::ChoiceState,
};

use crate::logic::choice_eval::eval_choice_logic;

/// Evaluate `ChoiceState` and return原上下文.
///
/// * 若任一 `choices` 条件满足，直接 `Ok(ctx.clone())`
/// * 若无匹配但存在 `default` 分支，同样返回 `ctx.clone()`
/// * 否则报错
pub async fn handle_choice(
    state_name: &str,
    state: &ChoiceState,
    ctx: &Value,
) -> Result<Value, String> {
    // 遍历显式 choices
    for ChoiceRule { condition, .. } in &state.choices {
        if eval_choice_logic(condition, ctx)? {
            return Ok(ctx.clone());
        }
    }

    // 若没有命中但 DSL 提供 default
    if state.default.is_some() {
        return Ok(ctx.clone());
    }

    // 否则报错
    Err(format!("No matching branch in choice '{}'.", state_name))
}
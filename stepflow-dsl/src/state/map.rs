use serde::{Deserialize, Serialize};

use super::base::BaseState;
use crate::branch::Branch;

/// `MapState` 表示一个迭代型状态节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapState {
    /// 通用基础字段：输入输出映射、next、end、retry、catch 等
    #[serde(flatten)]
    pub base: BaseState,

    /// 表示要从父上下文中提取的列表路径，如 `$.users`
    pub items_path: String,

    /// 迭代流程的子工作流定义（即 iterator）
    pub iterator: Branch,

    /// 控制并发的最大子流程数量
    #[serde(default)]
    pub max_concurrency: Option<u32>,

    /// 可选：将每个 item 放入的上下文 key，默认为 "item"
    #[serde(default = "default_item_context_key")]
    pub item_context_key: String,
}

/// 默认中间变量 key 是 "item"
fn default_item_context_key() -> String {
    "item".to_string()
}
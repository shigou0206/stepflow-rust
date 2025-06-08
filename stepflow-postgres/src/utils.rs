use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Patch<T> {
    Keep,         // 不修改
    Null,         // 显式设置为 NULL
    Value(T),     // 设置为某个具体值
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Patch::Keep
    }
}
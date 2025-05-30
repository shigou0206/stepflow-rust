use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

use super::rule::MappingRule;

/// 输入字段保留策略
#[derive(Debug, Clone, Serialize)]
pub enum PreserveFields {
    /// 保留全部字段
    All,
    /// 不保留任何字段
    None,
    /// 仅保留指定字段列表
    Some(Vec<String>),
}

impl Default for PreserveFields {
    fn default() -> Self {
        PreserveFields::None
    }
}

struct PreserveFieldsVisitor;

impl<'de> Visitor<'de> for PreserveFieldsVisitor {
    type Value = PreserveFields;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string 'all'/'none' (case-insensitive) or a list of strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<PreserveFields, E>
    where
        E: de::Error,
    {
        match value.to_lowercase().as_str() {
            "all" => Ok(PreserveFields::All),
            "none" => Ok(PreserveFields::None),
            other => Err(de::Error::unknown_variant(other, &["all", "none"])),
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<PreserveFields, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = Vec::new();
        while let Some(elem) = seq.next_element()? {
            items.push(elem);
        }
        Ok(PreserveFields::Some(items))
    }
}

impl<'de> Deserialize<'de> for PreserveFields {
    fn deserialize<D>(deserializer: D) -> Result<PreserveFields, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PreserveFieldsVisitor)
    }
}

/// 顶层 DSL —— 对应一整个映射任务
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MappingDSL {
    /// 版本号（可选）
    pub version: Option<String>,

    /// 描述信息（可选）
    pub description: Option<String>,

    /// 输入字段保留策略：All / None（默认）/ Some([...])
    #[serde(default)]
    pub preserve: PreserveFields,

    /// 是否启用调试模式（默认 false）
    #[serde(default)]
    pub debug: bool,

    /// 映射规则列表
    pub mappings: Vec<MappingRule>,
}
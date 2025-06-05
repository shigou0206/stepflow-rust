use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

use super::rule::MappingRule;

/// 输入字段保留策略
#[derive(Debug, Clone, PartialEq)]
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

impl Serialize for PreserveFields {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PreserveFields::All => serializer.serialize_str("all"),
            PreserveFields::None => serializer.serialize_str("none"),
            PreserveFields::Some(list) => list.serialize(serializer),
        }
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
    /// 命名空间（可选）
    pub namespace: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_namespace_serde() {
        let dsl_json = json!({
            "namespace": "tenant_a",
            "version": "1.0",
            "description": "test",
            "preserve": "all",
            "debug": true,
            "mappings": []
        });

        let dsl: MappingDSL = serde_json::from_value(dsl_json.clone()).unwrap();
        assert_eq!(dsl.namespace.as_deref(), Some("tenant_a"));
        assert_eq!(dsl.version.as_deref(), Some("1.0"));
        assert_eq!(dsl.preserve, PreserveFields::All);
        assert!(dsl.debug);
        assert!(dsl.mappings.is_empty());

        let out = serde_json::to_value(&dsl).unwrap();
        assert_eq!(out["namespace"], "tenant_a");
        assert_eq!(out["preserve"], "all");
    }

    #[test]
    fn test_default_values() {
        let dsl: MappingDSL = serde_json::from_str("{\"mappings\":[]}").unwrap();
        assert_eq!(dsl.namespace, None);
        assert_eq!(dsl.preserve, PreserveFields::None);
        assert!(!dsl.debug);
        assert!(dsl.mappings.is_empty());
    }

    #[test]
    fn test_roundtrip() {
        let dsl = MappingDSL {
            namespace: Some("ns1".to_string()),
            version: Some("v1".to_string()),
            description: Some("desc".to_string()),
            preserve: PreserveFields::Some(vec!["foo".to_string(), "bar".to_string()]),
            debug: false,
            mappings: vec![],
        };
        let ser = serde_json::to_string(&dsl).unwrap();
        let de: MappingDSL = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.namespace, Some("ns1".to_string()));
        assert_eq!(de.preserve, PreserveFields::Some(vec!["foo".to_string(), "bar".to_string()]));
    }
}
use serde_json::{Map, Value};
use jsonpath_lib::select;

use crate::{
    error::{MappingError, Result},
    model::rule::MappingRule,
    resolver,
    utils::merge_value,
};

/// 对数组或单个对象执行子映射
pub fn resolve_submapping(rule: &MappingRule, input: &Value) -> Result<Value> {
    // 1️⃣ 取出 source 路径
    let path = rule
        .source
        .as_ref()
        .ok_or_else(|| MappingError::MissingField("source"))?;

    // 2️⃣ 用 JSONPath 找到所有节点
    let raw_nodes = select(input, path).map_err(|e| MappingError::JsonPath(e.to_string()))?;

    // 3️⃣ 如果某个节点本身是数组，把它拆成多个元素
    let mut elems = Vec::new();
    for node_ref in raw_nodes {
        let node = node_ref.clone();
        if let Value::Array(arr) = node {
            for item in arr {
                elems.push(item);
            }
        } else {
            elems.push(node);
        }
    }

    // 4️⃣ 没节点就空数组返回
    if elems.is_empty() {
        return Ok(Value::Array(vec![]));
    }

    // 5️⃣ 依次对每个 element 运行子规则
    let subs = rule
        .sub_mappings
        .as_ref()
        .ok_or_else(|| MappingError::MissingField("subMappings"))?;

    let mut results = Vec::new();
    for elem in elems {
        let mut obj = Map::new();
        for sub in subs {
            // 关键：这里传入的就是 elem（单个对象），而不是顶层 input
            let val = resolver::resolve(sub, &elem)?;
            merge_value(&mut obj, &sub.key, val, sub.merge_strategy);
        }
        results.push(Value::Object(obj));
    }

    Ok(Value::Array(results))
}
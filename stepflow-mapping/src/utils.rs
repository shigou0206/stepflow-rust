use serde_json::{Map, Value};

use crate::model::rule::MergeStrategy;

/// 写入嵌套 key：`user.name.first`
pub fn insert_nested(obj: &mut Map<String, Value>, path: &str, value: Value) {
    let mut parts = path.split('.').collect::<Vec<_>>();
    if parts.is_empty() {
        return;
    }
    let last = parts.pop().unwrap();
    let mut curr = obj;

    for p in parts {
        curr = curr
            .entry(p.to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("intermediate not object");
    }
    curr.insert(last.to_string(), value);
}

// -------- 对外 API --------
pub fn merge_value(out: &mut Map<String, Value>, key: &str, val: Value, strat: MergeStrategy) {
    match strat {
        /* 覆盖 */
        MergeStrategy::Overwrite => insert_nested(out, key, val),

        /* 仅在键不存在时写入 */
        MergeStrategy::Ignore => {
            if !out.contains_key(key) {
                insert_nested(out, key, val)
            }
        }

        /* 追加到数组 */
        MergeStrategy::Append => {
            // 若目标不存在或非数组，先初始化为空数组
            if !out.get(key).map_or(false, |v| v.is_array()) {
                insert_nested(out, key, Value::Array(vec![]));
            }
            if let Some(arr) = out.get_mut(key).and_then(Value::as_array_mut) {
                arr.push(val);
            }
        }

        /* 将对象字段合并到目标对象 */
        MergeStrategy::Merge => {
            if let Value::Object(obj_new) = val {
                // 若目标不存在或不是对象，先变为空对象
                if !out.get(key).map_or(false, |v| v.is_object()) {
                    insert_nested(out, key, Value::Object(Map::new()));
                }
                if let Some(Value::Object(target)) = out.get_mut(key) {
                    for (k, v) in obj_new {
                        target.insert(k, v);
                    }
                }
            } else {
                // 若传入不是对象，退回覆盖行为
                insert_nested(out, key, val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map};
    use crate::model::rule::MergeStrategy;

    #[test]
    fn test_insert_nested_simple() {
        let mut obj = Map::new();
        insert_nested(&mut obj, "a.b.c", json!(1));
        assert_eq!(obj["a"]["b"]["c"], 1);
    }

    #[test]
    fn test_merge_value_overwrite() {
        let mut obj = Map::new();
        merge_value(&mut obj, "foo", json!(1), MergeStrategy::Overwrite);
        merge_value(&mut obj, "foo", json!(2), MergeStrategy::Overwrite);
        assert_eq!(obj["foo"], 2);
    }

    #[test]
    fn test_merge_value_ignore() {
        let mut obj = Map::new();
        merge_value(&mut obj, "foo", json!(1), MergeStrategy::Ignore);
        merge_value(&mut obj, "foo", json!(2), MergeStrategy::Ignore);
        assert_eq!(obj["foo"], 1);
    }

    #[test]
    fn test_merge_value_append() {
        let mut obj = Map::new();
        merge_value(&mut obj, "arr", json!(1), MergeStrategy::Append);
        merge_value(&mut obj, "arr", json!(2), MergeStrategy::Append);
        assert_eq!(obj["arr"], json!([1, 2]));
    }

    #[test]
    fn test_merge_value_merge_object() {
        let mut obj = Map::new();
        merge_value(&mut obj, "obj", json!({"a": 1}), MergeStrategy::Merge);
        merge_value(&mut obj, "obj", json!({"b": 2}), MergeStrategy::Merge);
        assert_eq!(obj["obj"], json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_value_merge_non_object() {
        let mut obj = Map::new();
        merge_value(&mut obj, "obj", json!({"a": 1}), MergeStrategy::Merge);
        merge_value(&mut obj, "obj", json!(42), MergeStrategy::Merge);
        assert_eq!(obj["obj"], 42);
    }
}
use std::collections::{HashMap, VecDeque};

use crate::{
    error::{MappingError, Result},
    model::rule::MappingRule,
};

/// 拓扑排序；若存在依赖则按 depends_on 排序，若有环返回 `MappingError::CircularDependency`
pub fn sort_rules(rules: &[MappingRule]) -> Result<Vec<MappingRule>> {
    // 如果所有规则都没有 dependsOn，直接按原始顺序返回
    if rules.iter().all(|r| r.depends_on.as_ref().map_or(true, |v| v.is_empty())) {
        return Ok(rules.to_vec());
    }

    // 1. 把 key 映射到 rule
    let mut map: HashMap<String, &MappingRule> = HashMap::new();
    for r in rules {
        map.insert(r.key.clone(), r);
    }

    // 2. 构建邻接表 & 入度表
    let mut indeg: HashMap<String, usize> = HashMap::new();
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();

    for r    in rules {
        let deps = r.depends_on.as_ref().cloned().unwrap_or_default();
        indeg.entry(r.key.clone()).or_insert(0);

        for d in deps {
            edges.entry(d.clone()).or_default().push(r.key.clone());
            *indeg.entry(r.key.clone()).or_insert(0) += 1;
        }
    }

    // 3. Kahn 算法
    let mut queue: VecDeque<String> =
        indeg.iter().filter(|(_, v)| **v == 0).map(|(k, _)| k.clone()).collect();

    let mut output_keys = Vec::<String>::new();
    while let Some(k) = queue.pop_front() {
        output_keys.push(k.clone());
        if let Some(nexts) = edges.get(&k) {
            for n in nexts {
                let e = indeg.get_mut(n).unwrap();
                *e -= 1;
                if *e == 0 {
                    queue.push_back(n.clone());
                }
            }
        }
    }

    // 检测环
    if output_keys.len() != rules.len() {
        return Err(MappingError::CircularDependency);
    }

    // 4. 按拓扑顺序收集 rule
    let ordered = output_keys
        .into_iter()
        .filter_map(|k| map.get(&k).cloned().cloned())
        .collect();

    Ok(ordered)
}

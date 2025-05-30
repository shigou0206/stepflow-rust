use jsonpath_lib::select;
use serde_json::Value;
use stepflow_dsl::logic::ChoiceLogic;

pub fn eval_choice_logic(logic: &ChoiceLogic, data: &Value) -> Result<bool, String> {
    // And
    if let Some(and) = &logic.and_ {
        for cond in and {
            if !eval_choice_logic(cond, data)? {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    // Or
    if let Some(or) = &logic.or_ {
        for cond in or {
            if eval_choice_logic(cond, data)? {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    // Not
    if let Some(not) = &logic.not_ {
        return Ok(!eval_choice_logic(not, data)?);
    }

    // Leaf comparison
    let variable = logic
        .variable
        .as_deref()
        .ok_or("Missing variable")?;
    let operator = logic
        .operator
        .as_deref()
        .ok_or("Missing operator")?;
    let cmp_value = logic.value.clone().unwrap_or(Value::Null);

    let selected = select(data, variable)
        .map_err(|e| format!("Invalid JSONPath '{}': {e}", variable))?
        .first()
        .cloned()
        .cloned()
        .unwrap_or(Value::Null);

    Ok(match operator {
        "Equals"    => selected == cmp_value,
        "NotEquals" => selected != cmp_value,

        // ↓ 数值比较（需都能转为 f64）
        "GreaterThan"        => num_cmp(&selected, &cmp_value, |a, b| a >  b)?,
        "GreaterThanEquals"  => num_cmp(&selected, &cmp_value, |a, b| a >= b)?,
        "LessThan"           => num_cmp(&selected, &cmp_value, |a, b| a <  b)?,
        "LessThanEquals"     => num_cmp(&selected, &cmp_value, |a, b| a <= b)?,

        // 类型判断
        "IsNull"     => selected.is_null(),
        "IsString"   => selected.is_string(),
        "IsBoolean"  => selected.is_boolean(),
        "IsNumeric"  => selected.is_number(),

        _ => return Err(format!("Unsupported operator {operator}")),
    })
}

// ---------- helper ----------
fn num_cmp<F>(a: &Value, b: &Value, op: F) -> Result<bool, String>
where
    F: Fn(f64, f64) -> bool,
{
    let fa = a.as_f64().ok_or("Left side not numeric")?;
    let fb = b.as_f64().ok_or("Right side not numeric")?;
    Ok(op(fa, fb))
}
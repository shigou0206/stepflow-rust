use crate::registry::ERROR_REGISTRY;
use stepflow_dto::dto::error_policy::{RetryPolicy, CatchPolicy};

/// 判断 error_type 是否已注册
pub fn is_valid_error_type(error_type: &str) -> bool {
    if error_type == "*" {
        return true;
    }
    ERROR_REGISTRY.lock().unwrap().contains_key(error_type)
}

/// 验证 Retry 列表中所有 error_equals 是否合法
pub fn validate_retry_list(policies: &[RetryPolicy]) -> Result<(), String> {
    for policy in policies {
        for err in &policy.error_equals {
            if !is_valid_error_type(err) {
                return Err(format!("Invalid error_type in Retry: {}", err));
            }
        }
    }
    Ok(())
}

/// 验证 Catch 列表中所有 error_equals 是否合法
pub fn validate_catch_list(policies: &[CatchPolicy]) -> Result<(), String> {
    for policy in policies {
        for err in &policy.error_equals {
            if !is_valid_error_type(err) {
                return Err(format!("Invalid error_type in Catch: {}", err));
            }
        }
    }
    Ok(())
}
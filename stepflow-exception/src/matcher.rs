use stepflow_dto::dto::error_policy::{RetryPolicy, CatchPolicy};

/// 匹配最先符合的 Retry 策略（如多个）
/// error_type 可为 `"HttpTimeout"`、`"MappingJsonPathError"` 等
pub fn match_retry<'a>(
    error_type: &str,
    retry_policies: Option<&'a [RetryPolicy]>
) -> Option<&'a RetryPolicy> {
    retry_policies?.iter().find(|policy| {
        policy.error_equals.iter().any(|eq| eq == "*" || eq == error_type)
    })
}

/// 匹配最先符合的 Catch 策略
pub fn match_catch<'a>(
    error_type: &str,
    catch_policies: Option<&'a [CatchPolicy]>
) -> Option<&'a CatchPolicy> {
    catch_policies?.iter().find(|policy| {
        policy.error_equals.iter().any(|eq| eq == "*" || eq == error_type)
    })
}
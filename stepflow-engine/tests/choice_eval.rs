use serde_json::json;
use stepflow_engine::logic::choice_eval::eval_choice_logic;
use stepflow_dsl::logic::ChoiceLogic;

#[test]
fn test_choice_equals() {
    let logic = ChoiceLogic {
        variable: Some("$.a".into()),
        operator: Some("Equals".into()),
        value: Some(json!(1)),
        and_: None,
        or_: None,
        not_: None
    };
    let data = json!({ "a": 1 });
    assert!(eval_choice_logic(&logic, &data).unwrap());
}
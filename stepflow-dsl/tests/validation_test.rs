use stepflow_dsl::{WorkflowDSL, ValidationError};
use std::collections::HashMap;
use serde_json::json;

#[test]
fn test_valid_workflow() {
    let workflow_json = json!({
        "startAt": "FirstState",
        "states": {
            "FirstState": {
                "type": "task",
                "resource": "test-resource",
                "end": true
            }
        }
    });

    let workflow: WorkflowDSL = serde_json::from_value(workflow_json).unwrap();
    assert!(workflow.validate().is_ok());
}

#[test]
fn test_invalid_start_state() {
    let workflow_json = json!({
        "startAt": "NonExistentState",
        "states": {
            "FirstState": {
                "type": "task",
                "resource": "test-resource",
                "end": true
            }
        }
    });

    let workflow: WorkflowDSL = serde_json::from_value(workflow_json).unwrap();
    match workflow.validate() {
        Err(ValidationError::StartStateNotFound(_)) => (),
        _ => panic!("Expected StartStateNotFound error"),
    }
}

#[test]
fn test_invalid_next_state() {
    let workflow_json = json!({
        "startAt": "FirstState",
        "states": {
            "FirstState": {
                "type": "task",
                "resource": "test-resource",
                "next": "NonExistentState"
            }
        }
    });

    let workflow: WorkflowDSL = serde_json::from_value(workflow_json).unwrap();
    match workflow.validate() {
        Err(ValidationError::NextStateNotFound(_)) => (),
        _ => panic!("Expected NextStateNotFound error"),
    }
}

#[test]
fn test_next_and_end_conflict() {
    let workflow_json = json!({
        "startAt": "FirstState",
        "states": {
            "FirstState": {
                "type": "task",
                "resource": "test-resource",
                "next": "SecondState",
                "end": true
            },
            "SecondState": {
                "type": "task",
                "resource": "test-resource",
                "end": true
            }
        }
    });

    let workflow: WorkflowDSL = serde_json::from_value(workflow_json).unwrap();
    match workflow.validate() {
        Err(ValidationError::NextAndEndConflict(_)) => (),
        _ => panic!("Expected NextAndEndConflict error"),
    }
}

#[test]
fn test_no_end_state() {
    let workflow_json = json!({
        "startAt": "FirstState",
        "states": {
            "FirstState": {
                "type": "task",
                "resource": "test-resource",
                "next": "SecondState"
            },
            "SecondState": {
                "type": "task",
                "resource": "test-resource",
                "next": "FirstState"
            }
        }
    });

    let workflow: WorkflowDSL = serde_json::from_value(workflow_json).unwrap();
    match workflow.validate() {
        Err(ValidationError::NoEndState) => (),
        _ => panic!("Expected NoEndState error"),
    }
} 
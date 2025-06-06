use crate::WorkflowDSL;
use crate::state::State;
use thiserror::Error;
use std::collections::HashSet;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Start state '{0}' not found in states")]
    StartStateNotFound(String),
    
    #[error("State '{0}' referenced in 'next' field not found")]
    NextStateNotFound(String),
    
    #[error("State '{0}' has both 'next' and 'end' fields set")]
    NextAndEndConflict(String),
    
    #[error("No end state found in workflow")]
    NoEndState,
    
    #[error("Invalid state type for '{0}': {1}")]
    InvalidStateType(String, String),
    
    #[error("Missing required field in state '{0}': {1}")]
    MissingRequiredField(String, String),
}

impl WorkflowDSL {
    /// Validates the workflow definition
    pub fn validate(&self) -> Result<(), ValidationError> {
        // 1. Verify start state exists
        if !self.states.contains_key(&self.start_at) {
            return Err(ValidationError::StartStateNotFound(self.start_at.clone()));
        }

        // 2. Track visited states to ensure all states are reachable
        let mut visited = HashSet::new();
        let mut has_end_state = false;

        // 3. Validate each state
        for (name, state) in &self.states {
            self.validate_state(name, state, &mut visited, &mut has_end_state)?;
        }

        // 4. Verify at least one end state exists
        if !has_end_state {
            return Err(ValidationError::NoEndState);
        }

        Ok(())
    }

    fn validate_state(
        &self,
        name: &str,
        state: &State,
        visited: &mut HashSet<String>,
        has_end_state: &mut bool,
    ) -> Result<(), ValidationError> {
        visited.insert(name.to_string());

        // Get base state for common validations
        let base = match state {
            State::Task(t) => &t.base,
            State::Pass(p) => &p.base,
            State::Wait(w) => &w.base,
            State::Choice(c) => &c.base,
            State::Succeed(s) => &s.base,
            State::Fail(f) => &f.base,
            State::Parallel(p) => &p.base,
            State::Map(m) => &m.base,
        };

        // Check next state exists if specified
        if let Some(next) = &base.next {
            if !self.states.contains_key(next) {
                return Err(ValidationError::NextStateNotFound(next.clone()));
            }
        }

        // Check for next/end conflict
        if base.next.is_some() && base.end.unwrap_or(false) {
            return Err(ValidationError::NextAndEndConflict(name.to_string()));
        }

        // Mark if this is an end state
        if base.end.unwrap_or(false) {
            *has_end_state = true;
        }

        // State-specific validations
        match state {
            State::Task(task) => {
                if task.resource.is_empty() {
                    return Err(ValidationError::MissingRequiredField(
                        name.to_string(),
                        "resource".to_string(),
                    ));
                }
            }
            State::Choice(choice) => {
                if choice.choices.is_empty() && choice.default.is_none() {
                    return Err(ValidationError::MissingRequiredField(
                        name.to_string(),
                        "choices or default".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
} 
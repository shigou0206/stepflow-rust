// ✅ step_once：支持 Map / Parallel 状态生成命令
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use stepflow_dsl::{WorkflowDSL, State};
use crate::command::Command;
use crate::logic::choice_eval::eval_choice_logic;

pub fn step_once(
    dsl: &WorkflowDSL,
    state_name: &str,
    context: &Value,
) -> Result<Command, String> {
    let state = dsl.states.get(state_name)
        .ok_or_else(|| format!("State '{}' not found", state_name))?;

    match state {
        State::Task(task) => Ok(Command::ExecuteTask {
            state_name: state_name.to_string(),
            resource: task.resource.clone(),
            next_state: task.base.next.clone(),
        }),

        State::Wait(wait) => {
            let now = Utc::now();
            let wait_until = if let Some(seconds) = wait.seconds {
                now + Duration::seconds(seconds as i64)
            } else if let Some(ts) = &wait.timestamp {
                DateTime::parse_from_rfc3339(ts).map_err(|e| e.to_string())?.with_timezone(&Utc)
            } else {
                return Err("WaitState must define Seconds or Timestamp".to_string());
            };

            let seconds = (wait_until - now).num_seconds().max(0) as u64;

            Ok(Command::Wait {
                state_name: state_name.to_string(),
                seconds,
                wait_until,
                next_state: wait.base.next.clone(),
            })
        }

        State::Pass(pass) => {
            let result = pass.result.clone().unwrap_or_else(|| Value::Object(Default::default()));
            if pass.base.end.unwrap_or(false) && pass.base.next.is_none() {
                Ok(Command::Succeed {
                    state_name: state_name.to_string(),
                    output: result,
                })
            } else {
                Ok(Command::Pass {
                    state_name: state_name.to_string(),
                    output: result,
                    next_state: pass.base.next.clone(),
                })
            }
        }

        State::Succeed(_) => Ok(Command::Succeed {
            state_name: state_name.to_string(),
            output: context.clone(),
        }),

        State::Fail(fail) => Ok(Command::Fail {
            state_name: state_name.to_string(),
            error: fail.error.clone(),
            cause: fail.cause.clone(),
        }),

        State::Choice(choice) => {
            for branch in &choice.choices {
                if eval_choice_logic(&branch.condition, context)? {
                    return Ok(Command::Choice {
                        state_name: state_name.to_string(),
                        next_state: branch.next.clone(),
                    });
                }
            }
            if let Some(default_next) = &choice.default_next {
                Ok(Command::Choice {
                    state_name: state_name.to_string(),
                    next_state: default_next.clone(),
                })
            } else {
                Err(format!("No matching choice and no default in state '{}'.", state_name))
            }
        }

        State::Map(map) => Ok(Command::Map {
            state_name: state_name.to_string(),
            next_state: map.base.next.clone(),
        }),

        State::Parallel(par) => Ok(Command::Parallel {
            state_name: state_name.to_string(),
            next_state: par.base.next.clone(),
        }),
    }
}
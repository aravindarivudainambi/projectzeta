//! Cron scheduler that auto-executes agents with `Schedule` triggers.
//!
//! A background Tokio task polls all registered agents every 30 seconds.
//! When an agent's cron expression matches the current minute (and hasn't
//! already fired in this minute), a new run is created and executed using
//! the same `execute_run` infrastructure as manual runs.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{Datelike, Timelike, Utc};
use core_types::agent::{AgentConfig, AgentStep, Trigger};
use core_types::run::{AgentRun, RunStatus};
use serde_json::Value;
use uuid::Uuid;

use crate::state::{AppState, RunRecord};

/// Spawns the scheduler background task.
///
/// This function returns immediately. The task runs until the process exits.
pub fn spawn_scheduler(state: AppState) {
    tokio::spawn(async move {
        // Track last-fired time per agent to avoid double-fires within the same minute.
        let mut last_fired: HashMap<Uuid, chrono::DateTime<Utc>> = HashMap::new();

        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let now = Utc::now();

            let agents: Vec<(Uuid, AgentConfig)> = {
                let lock = state.agents.read().await;
                lock.iter().map(|(k, v)| (*k, v.clone())).collect()
            };

            for (agent_id, config) in agents {
                if let Trigger::Schedule { ref cron } = config.trigger {
                    if should_fire(cron, now, last_fired.get(&agent_id)) {
                        last_fired.insert(agent_id, now);
                        let state_clone = state.clone();
                        let config_clone = config.clone();
                        tokio::spawn(async move {
                            auto_execute_agent(state_clone, config_clone).await;
                        });
                    }
                }
            }
        }
    });
}

/// Determines whether a cron expression should fire at the given time.
///
/// Returns `false` if we already fired within the last 60 seconds for this agent.
fn should_fire(
    cron: &str,
    now: chrono::DateTime<Utc>,
    last_fired: Option<&chrono::DateTime<Utc>>,
) -> bool {
    if let Some(last) = last_fired {
        if now.signed_duration_since(*last).num_seconds() < 60 {
            return false;
        }
    }
    cron_matches(cron, now)
}

/// Simple five-field cron matcher: `minute hour dom month dow`.
///
/// Supports:
/// - `*` — matches any value
/// - A single number — exact match
/// - Comma-separated values — e.g. `1,15` matches 1 or 15
///
/// Enough for common patterns:
/// - `0 9 * * 1` — Monday at 9:00 AM
/// - `0 9 * * *` — Daily at 9:00 AM
/// - `0 * * * *` — Every hour on the hour
/// - `*/5 * * * *` — Every 5 minutes (step syntax)
fn cron_matches(cron: &str, now: chrono::DateTime<Utc>) -> bool {
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() != 5 {
        return false;
    }

    let minute = now.minute();
    let hour = now.hour();
    let dom = now.day();
    let month = now.month();
    // chrono: Monday = 0 .. Sunday = 6 for weekday().num_days_from_monday()
    // cron: Sunday = 0 .. Saturday = 6 (standard), or Sunday = 7
    let dow = now.weekday().num_days_from_sunday();

    field_matches(fields[0], minute)
        && field_matches(fields[1], hour)
        && field_matches(fields[2], dom)
        && field_matches(fields[3], month)
        && field_matches_dow(fields[4], dow)
}

/// Checks if a single cron field matches a value.
fn field_matches(field: &str, value: u32) -> bool {
    if field == "*" {
        return true;
    }

    // Step syntax: */N
    if let Some(step) = field.strip_prefix("*/") {
        if let Ok(n) = step.parse::<u32>() {
            return n > 0 && value % n == 0;
        }
    }

    // Comma-separated list
    for part in field.split(',') {
        if let Ok(n) = part.trim().parse::<u32>() {
            if n == value {
                return true;
            }
        }
    }

    false
}

/// Matches the day-of-week field, handling both 0=Sunday and 7=Sunday conventions.
fn field_matches_dow(field: &str, dow_sunday_zero: u32) -> bool {
    if field == "*" {
        return true;
    }

    if let Some(step) = field.strip_prefix("*/") {
        if let Ok(n) = step.parse::<u32>() {
            return n > 0 && dow_sunday_zero % n == 0;
        }
    }

    for part in field.split(',') {
        if let Ok(n) = part.trim().parse::<u32>() {
            // Accept both 0 and 7 as Sunday.
            let normalized = if n == 7 { 0 } else { n };
            if normalized == dow_sunday_zero {
                return true;
            }
        }
    }

    false
}

/// Creates a run from the agent's config and executes it headlessly (no SSE).
async fn auto_execute_agent(state: AppState, config: AgentConfig) {
    let run_id = Uuid::new_v4();
    let agent_id = config.id;

    let mut tool_bindings: HashMap<Uuid, (String, Value)> = HashMap::new();
    let steps: Vec<AgentStep> = config
        .steps
        .iter()
        .map(|s| {
            let step = AgentStep {
                id: Uuid::new_v4(),
                name: s.name.clone(),
                tool_name: s.tool_name.clone(),
                requires_approval: false, // Scheduled runs skip approval gates
            };
            if let Some(ref tool_name) = s.tool_name {
                tool_bindings.insert(step.id, (tool_name.clone(), Value::Object(Default::default())));
            }
            step
        })
        .collect();

    let record = RunRecord {
        run: AgentRun {
            id: run_id,
            agent_id,
            status: RunStatus::Running,
        },
        steps: steps.clone(),
        tool_bindings: tool_bindings.clone(),
    };

    {
        let mut runs = state.runs.write().await;
        runs.insert(run_id, record);
    }

    crate::routes::runs::execute_run_headless(run_id, agent_id, steps, tool_bindings, state).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn cron_every_minute() {
        let now = Utc.with_ymd_and_hms(2026, 3, 7, 10, 30, 0).unwrap();
        assert!(cron_matches("* * * * *", now));
    }

    #[test]
    fn cron_specific_time() {
        let now = Utc.with_ymd_and_hms(2026, 3, 7, 9, 0, 0).unwrap();
        assert!(cron_matches("0 9 * * *", now));
        assert!(!cron_matches("0 10 * * *", now));
    }

    #[test]
    fn cron_monday_only() {
        // March 9, 2026 is a Monday.
        let monday = Utc.with_ymd_and_hms(2026, 3, 9, 9, 0, 0).unwrap();
        assert!(cron_matches("0 9 * * 1", monday));

        // March 10, 2026 is a Tuesday.
        let tuesday = Utc.with_ymd_and_hms(2026, 3, 10, 9, 0, 0).unwrap();
        assert!(!cron_matches("0 9 * * 1", tuesday));
    }

    #[test]
    fn cron_step_syntax() {
        let now = Utc.with_ymd_and_hms(2026, 3, 7, 10, 15, 0).unwrap();
        assert!(cron_matches("*/5 * * * *", now));
        assert!(!cron_matches("*/7 * * * *", now));
    }

    #[test]
    fn should_fire_debounces() {
        let now = Utc.with_ymd_and_hms(2026, 3, 7, 9, 0, 30).unwrap();
        let last = Utc.with_ymd_and_hms(2026, 3, 7, 9, 0, 0).unwrap();
        assert!(!should_fire("0 9 * * *", now, Some(&last)));

        let later = Utc.with_ymd_and_hms(2026, 3, 7, 9, 1, 5).unwrap();
        // Different minute but cron says minute=0, so cron doesn't match at minute=1.
        assert!(!should_fire("0 9 * * *", later, Some(&last)));
    }
}

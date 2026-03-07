use std::{collections::VecDeque, time::Instant};

use anyhow::{Context, Result};
use core_types::{
    agent::{AgentConfig, AgentStep},
    events::AgentEvent,
};
use serde_json::{json, Value};
use tokio::sync::mpsc;

/// Represents a planner-produced execution unit for the current run loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedStep {
    pub name: String,
}

/// Defines the planner boundary used by the run loop.
///
/// The planner decides which step should execute next and exposes whether the
/// run has completed. This intentionally remains minimal until real planning
/// context and traces are added.
pub trait Planner {
    fn next_step(&mut self, config: &AgentConfig) -> Result<Option<PlannedStep>>;
    fn is_done(&self) -> bool;
}

/// Defines the tool invocation boundary used by the run loop.
///
/// The runner only relies on a typed JSON result, allowing connector-hub and
/// policy checks to evolve independently.
pub trait ToolCaller {
    fn invoke(&mut self, step: &PlannedStep) -> Result<Value>;
}

/// Runs an agent from planning through completion and emits structured events.
///
/// Execution order:
/// 1. Request next step from planner.
/// 2. Invoke tool caller with that step.
/// 3. Emit `StepCompleted`.
/// 4. Check planner completion and continue or finish.
/// 5. Emit `RunFinished` once at the end.
pub async fn run(config: AgentConfig, event_tx: mpsc::Sender<AgentEvent>) -> Result<()> {
    let mut planner = SequentialPlanner::from_steps(config.steps.clone());
    let mut tool_caller = DefaultToolCaller;
    run_with(config, event_tx, &mut planner, &mut tool_caller).await
}

/// Executes the run loop with injected planner/tool-caller dependencies.
///
/// This exists primarily to keep production orchestration logic testable with
/// deterministic mocks.
pub async fn run_with<P, T>(
    config: AgentConfig,
    event_tx: mpsc::Sender<AgentEvent>,
    planner: &mut P,
    tool_caller: &mut T,
) -> Result<()>
where
    P: Planner,
    T: ToolCaller,
{
    loop {
        let Some(step) = planner.next_step(&config)? else {
            break;
        };

        let started_at = Instant::now();
        let result = tool_caller
            .invoke(&step)
            .with_context(|| format!("tool invocation failed for step '{}'", step.name))?;

        event_tx
            .send(AgentEvent::StepCompleted {
                result,
                latency_ms: started_at.elapsed().as_millis() as u64,
            })
            .await
            .context("failed to emit StepCompleted event")?;

        if planner.is_done() {
            break;
        }
    }

    event_tx
        .send(AgentEvent::RunFinished { cost_usd: 0.0 })
        .await
        .context("failed to emit RunFinished event")?;

    Ok(())
}

/// Provides a deterministic planner that drains `AgentConfig.steps` in order.
#[derive(Debug, Default)]
struct SequentialPlanner {
    queue: VecDeque<PlannedStep>,
}

impl SequentialPlanner {
    fn from_steps(steps: Vec<AgentStep>) -> Self {
        let queue = steps
            .into_iter()
            .map(|step| PlannedStep { name: step.name })
            .collect();
        Self { queue }
    }
}

impl Planner for SequentialPlanner {
    fn next_step(&mut self, _config: &AgentConfig) -> Result<Option<PlannedStep>> {
        Ok(self.queue.pop_front())
    }

    fn is_done(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Provides a deterministic tool-caller result shape for scaffolded execution.
#[derive(Debug, Default)]
struct DefaultToolCaller;

impl ToolCaller for DefaultToolCaller {
    fn invoke(&mut self, step: &PlannedStep) -> Result<Value> {
        Ok(json!({
            "status": "ok",
            "step": step.name,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{run_with, PlannedStep, Planner, ToolCaller};
    use anyhow::Result;
    use core_types::{agent::sample_agent_config, events::AgentEvent};
    use serde_json::{json, Value};
    use std::collections::VecDeque;
    use tokio::sync::mpsc;

    #[derive(Debug)]
    struct MockPlanner {
        queue: VecDeque<PlannedStep>,
    }

    impl MockPlanner {
        fn from_names(names: &[&str]) -> Self {
            let queue = names
                .iter()
                .map(|name| PlannedStep {
                    name: (*name).to_string(),
                })
                .collect();
            Self { queue }
        }
    }

    impl Planner for MockPlanner {
        fn next_step(
            &mut self,
            _config: &core_types::agent::AgentConfig,
        ) -> Result<Option<PlannedStep>> {
            Ok(self.queue.pop_front())
        }

        fn is_done(&self) -> bool {
            self.queue.is_empty()
        }
    }

    #[derive(Debug, Default)]
    struct MockToolCaller;

    impl ToolCaller for MockToolCaller {
        fn invoke(&mut self, step: &PlannedStep) -> Result<Value> {
            Ok(json!({ "step_name": step.name }))
        }
    }

    #[tokio::test]
    async fn emits_three_step_completed_events_in_order_then_run_finished() {
        let config = sample_agent_config();
        let (tx, mut rx) = mpsc::channel(16);
        let mut planner = MockPlanner::from_names(&["first", "second", "third"]);
        let mut tool_caller = MockToolCaller;

        run_with(config, tx, &mut planner, &mut tool_caller)
            .await
            .expect("run loop should succeed");

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        assert_eq!(events.len(), 4, "expected exactly 4 events");

        match &events[0] {
            AgentEvent::StepCompleted { result, .. } => {
                assert_eq!(result, &json!({ "step_name": "first" }));
            }
            other => panic!("expected StepCompleted for event 0, got {other:?}"),
        }

        match &events[1] {
            AgentEvent::StepCompleted { result, .. } => {
                assert_eq!(result, &json!({ "step_name": "second" }));
            }
            other => panic!("expected StepCompleted for event 1, got {other:?}"),
        }

        match &events[2] {
            AgentEvent::StepCompleted { result, .. } => {
                assert_eq!(result, &json!({ "step_name": "third" }));
            }
            other => panic!("expected StepCompleted for event 2, got {other:?}"),
        }

        assert!(
            matches!(events[3], AgentEvent::RunFinished { .. }),
            "last event should be RunFinished"
        );
    }
}

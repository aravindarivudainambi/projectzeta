/// Represents the event categories emitted while an agent run progresses.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    StepStarted,
    ToolCalled,
    HumanApproval,
    Finished,
}

/// Returns a short display label for an engine event.
pub fn event_label(event: &AgentEvent) -> &'static str {
    match event {
        AgentEvent::StepStarted => "step_started",
        AgentEvent::ToolCalled => "tool_called",
        AgentEvent::HumanApproval => "human_approval",
        AgentEvent::Finished => "finished",
    }
}

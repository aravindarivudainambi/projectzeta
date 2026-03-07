use anyhow::Result;

/// Starts a new agent run on behalf of the authenticated user.
pub async fn create_run() -> Result<()> {
    todo!("Validate execution permissions and enqueue a new agent run.")
}

/// Streams run events to subscribed clients using server-sent events.
pub async fn stream_run() -> Result<()> {
    todo!("Bridge the event source from the agent engine into an SSE response.")
}

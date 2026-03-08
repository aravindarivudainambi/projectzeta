const API_BASE =
  typeof process !== "undefined" && process.env?.NEXT_PUBLIC_API_BASE_URL
    ? process.env.NEXT_PUBLIC_API_BASE_URL
    : "http://localhost:8080";

export interface CreateRunPayload {
  agent_id: string;
  steps: Array<{
    name: string;
    requires_approval: boolean;
    tool_name?: string;
    tool_arguments?: Record<string, unknown>;
  }>;
}

export interface CreateRunResponse {
  run_id: string;
  status: string;
}

/**
 * Creates a new agent run. The run starts in Pending status and must be
 * streamed via GET /runs/{id}/stream to begin execution.
 */
export async function startRun(
  payload: CreateRunPayload,
): Promise<CreateRunResponse> {
  const res = await fetch(`${API_BASE}/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error(`startRun failed: ${res.status}`);
  return res.json();
}

/**
 * Returns the absolute SSE stream URL for a given run.
 */
export function streamRunUrl(runId: string): string {
  return `${API_BASE}/runs/${runId}/stream`;
}

/**
 * Approves a pending human approval checkpoint for a run.
 */
export async function approveRun(runId: string): Promise<void> {
  const res = await fetch(`${API_BASE}/runs/${runId}/approve`, {
    method: "POST",
  });
  if (!res.ok) throw new Error(`approveRun failed: ${res.status}`);
}

/**
 * Rejects a pending human approval checkpoint for a run.
 */
export async function rejectRun(runId: string): Promise<void> {
  const res = await fetch(`${API_BASE}/runs/${runId}/reject`, {
    method: "POST",
  });
  if (!res.ok) throw new Error(`rejectRun failed: ${res.status}`);
}

import type { AgentConfig, RunHistoryEntry, Trigger } from "@schema-types";

const API_BASE =
  typeof process !== "undefined" && process.env?.NEXT_PUBLIC_API_BASE_URL
    ? process.env.NEXT_PUBLIC_API_BASE_URL
    : "http://localhost:8080";

export interface AgentTokenResponse {
  token: string;
}

export interface CreateAgentPayload {
  name: string;
  trigger: Trigger;
  steps: Array<{
    name: string;
    tool_name?: string;
    requires_approval: boolean;
  }>;
}

/**
 * Issues a tenant-scoped JWT for an agent.
 */
export async function issueAgentToken(
  agentId: string,
): Promise<AgentTokenResponse> {
  const res = await fetch(`${API_BASE}/agents/${agentId}/token`, {
    method: "POST",
  });
  if (!res.ok) throw new Error(`issueAgentToken failed: ${res.status}`);
  return res.json();
}

/**
 * Lists agents visible to the current user.
 */
export async function listAgents(): Promise<AgentConfig[]> {
  const res = await fetch(`${API_BASE}/agents`);
  if (!res.ok) throw new Error(`listAgents failed: ${res.status}`);
  return res.json();
}

/**
 * Persists a new agent configuration.
 */
export async function createAgent(
  payload: CreateAgentPayload,
): Promise<AgentConfig> {
  const res = await fetch(`${API_BASE}/agents`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });

  if (!res.ok) throw new Error(`createAgent failed: ${res.status}`);
  return res.json();
}

/**
 * Fetches a single agent by identifier.
 */
export async function getAgent(agentId: string): Promise<AgentConfig | null> {
  const res = await fetch(`${API_BASE}/agents/${agentId}`);

  if (res.status === 404) {
    return null;
  }

  if (!res.ok) throw new Error(`getAgent failed: ${res.status}`);
  return res.json();
}

/**
 * Lists run history entries for a specific agent, most recent first.
 */
export async function listAgentRuns(
  agentId: string,
): Promise<RunHistoryEntry[]> {
  const res = await fetch(`${API_BASE}/agents/${agentId}/runs`);
  if (!res.ok) throw new Error(`listAgentRuns failed: ${res.status}`);
  return res.json();
}

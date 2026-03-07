const API_BASE =
  typeof process !== "undefined" && process.env?.NEXT_PUBLIC_API_BASE_URL
    ? process.env.NEXT_PUBLIC_API_BASE_URL
    : "http://localhost:8080";

export interface AgentTokenResponse {
  token: string;
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
 *
 * Stub — no list endpoint exists on the backend yet.
 */
export async function listAgents() {
  return [];
}

/**
 * Fetches a single agent by identifier.
 *
 * Stub — no get endpoint exists on the backend yet.
 */
export async function getAgent(_agentId: string) {
  return null;
}

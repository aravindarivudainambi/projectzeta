import type { AgentConfig, MarketplaceTemplate } from "@schema-types";

const API_BASE =
  typeof process !== "undefined" && process.env?.NEXT_PUBLIC_API_BASE_URL
    ? process.env.NEXT_PUBLIC_API_BASE_URL
    : "http://localhost:8080";

export interface ForkMarketplaceTemplatePayload {
  templateId: string;
  name?: string;
}

/**
 * Lists curated templates available in the internal marketplace catalog.
 */
export async function listMarketplaceTemplates(): Promise<
  MarketplaceTemplate[]
> {
  const res = await fetch(`${API_BASE}/marketplace`, {
    cache: "no-store",
  });

  if (!res.ok) {
    throw new Error(`listMarketplaceTemplates failed: ${res.status}`);
  }

  return res.json();
}

/**
 * Forks a marketplace template into a saved agent owned by the current workspace.
 */
export async function forkMarketplaceTemplate(
  payload: ForkMarketplaceTemplatePayload,
): Promise<AgentConfig> {
  const res = await fetch(`${API_BASE}/marketplace/fork`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });

  if (!res.ok) {
    throw new Error(`forkMarketplaceTemplate failed: ${res.status}`);
  }

  return res.json();
}

const API_BASE =
  typeof process !== "undefined" && process.env?.NEXT_PUBLIC_API_BASE_URL
    ? process.env.NEXT_PUBLIC_API_BASE_URL
    : "http://localhost:8080";

export interface ConnectorInfo {
  name: string;
  display_name: string;
  connected: boolean;
}

export interface OAuthStartResponse {
  redirect_url: string;
}

/**
 * Lists available connectors with their connection status.
 */
export async function listConnectors(token: string): Promise<ConnectorInfo[]> {
  const res = await fetch(`${API_BASE}/connectors`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!res.ok) throw new Error(`listConnectors failed: ${res.status}`);
  return res.json();
}

/**
 * Gets the OAuth redirect URL for Notion.
 */
export async function getNotionOAuthUrl(
  token: string,
): Promise<OAuthStartResponse> {
  const res = await fetch(`${API_BASE}/connectors/notion/oauth-url`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!res.ok) throw new Error(`getNotionOAuthUrl failed: ${res.status}`);
  return res.json();
}

/**
 * Exchanges the OAuth callback code for a stored access token.
 */
export async function exchangeNotionCode(
  token: string,
  code: string,
): Promise<void> {
  const res = await fetch(`${API_BASE}/connectors/notion/callback`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ code }),
  });
  if (!res.ok) throw new Error(`exchangeNotionCode failed: ${res.status}`);
}

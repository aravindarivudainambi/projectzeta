import { listMarketplaceTemplates } from "@api-client";
import type { MarketplaceTemplate } from "@schema-types";

/**
 * Returns marketplace templates from the API gateway, falling back to an empty list
 * when the backend is unavailable so the app can still render a stable shell.
 */
export async function loadMarketplaceTemplates(): Promise<MarketplaceTemplate[]> {
  try {
    return await listMarketplaceTemplates();
  } catch {
    return [];
  }
}

/**
 * Temporary connected-tool snapshot used until tenant connector state is threaded
 * into the marketplace surface.
 */
export const userConnectedTools = ["Google Workspace", "Notion"];
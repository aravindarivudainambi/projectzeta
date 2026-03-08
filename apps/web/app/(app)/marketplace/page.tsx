import AgentsClient from "../agents/AgentsClient";
import { loadMarketplaceTemplates, userConnectedTools } from "@/lib/marketplace";

/**
 * Renders the backend-backed marketplace catalog.
 */
export default async function MarketplacePage() {
  const templates = await loadMarketplaceTemplates();

  return (
    <main className="h-full bg-white ">
      <AgentsClient
        initialTemplates={templates}
        userConnectedTools={userConnectedTools}
      />
    </main>
  );
}

import AgentsClient from "./AgentsClient";
import {
  loadMarketplaceTemplates,
  userConnectedTools,
} from "@/lib/marketplace";

/**
 * Renders the agents marketplace / index.
 *
 * This lists pre-built agent templates that users can browse and fork.
 */
export default async function AgentsPage() {
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

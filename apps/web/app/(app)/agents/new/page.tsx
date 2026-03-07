import { AgentPreview } from "@/components/agent-builder/AgentPreview";
import { NLBuilder } from "@/components/agent-builder/NLBuilder";
import { WorkflowCanvas } from "@/components/agent-builder/WorkflowCanvas";

/**
 * Renders the new agent builder workspace.
 *
 * The real implementation should coordinate natural-language authoring, visual editing,
 * validation, and live previews using shared schemas.
 */
export default function NewAgentPage() {
  return (
    <main>
      <NLBuilder />
      <WorkflowCanvas />
      <AgentPreview />
    </main>
  );
}

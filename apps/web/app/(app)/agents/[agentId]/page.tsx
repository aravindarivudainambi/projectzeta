import AgentDetailClient from "./AgentDetailClient";

interface AgentDetailPageProps {
  params: Promise<{ agentId: string }>;
}

export default async function AgentDetailPage({
  params,
}: AgentDetailPageProps) {
  const { agentId } = await params;
  return <AgentDetailClient agentId={agentId} />;
}

import { LiveRunViewer } from "@/components/run-viewer/LiveRunViewer";

interface RunDetailPageProps {
  params: Promise<{ runId: string }>;
}

/**
 * Renders the live run viewer for a specific run.
 *
 * When a real runId from the URL matches a backend run, the viewer subscribes
 * to SSE events. Otherwise it falls back to the built-in mock simulation.
 */
export default async function RunDetailPage({ params }: RunDetailPageProps) {
  const { runId } = await params;

  return (
    <main className="flex-1 w-full bg-black h-full overflow-hidden flex flex-col">
      <LiveRunViewer runId={runId} />
    </main>
  );
}

import { LiveRunViewer } from "@/components/run-viewer/LiveRunViewer";

/**
 * Renders the live run viewer route placeholder.
 *
 * This page should eventually subscribe to server-sent events and render run progress in real time.
 */
export default function RunDetailPage() {
  return (
    <main className="flex-1 w-full bg-black h-full overflow-hidden flex flex-col">
      <LiveRunViewer />
    </main>
  );
}

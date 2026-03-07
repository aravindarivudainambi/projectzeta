import { LiveRunViewer } from "@/components/run-viewer/LiveRunViewer";

/**
 * Renders the live run viewer route placeholder.
 *
 * This page should eventually subscribe to server-sent events and render run progress in real time.
 */
export default function RunDetailPage() {
  return (
    <main>
      <LiveRunViewer />
    </main>
  );
}

import { CostChart } from "@/components/observability/CostChart";
import { LatencyHeatmap } from "@/components/observability/LatencyHeatmap";

/**
 * Renders the global observability dashboard placeholder.
 *
 * The final server component should load aggregated telemetry and stream high-value summaries.
 */
export default function ObservabilityPage() {
  return (
    <main>
      <CostChart />
      <LatencyHeatmap />
    </main>
  );
}

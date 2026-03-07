"use client";

import { ApprovalModal } from "./ApprovalModal";
import { StepCard } from "./StepCard";

/**
 * Renders the live run viewer shell.
 *
 * This component should eventually subscribe to the run stream, fan events into UI state,
 * and coordinate any approval interruptions.
 */
export function LiveRunViewer() {
  return (
    <section>
      <StepCard />
      <ApprovalModal />
    </section>
  );
}

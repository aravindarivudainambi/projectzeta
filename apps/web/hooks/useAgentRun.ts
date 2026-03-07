"use client";

import { useEffect, useRef, useState } from "react";
import { createSseClient, SseClient } from "@/lib/sse";
import { streamRunUrl } from "@api-client";

/** Mirrors the Rust AgentEvent enum serialized as externally-tagged JSON. */
export type AgentEvent =
  | { StepStarted: { step_id: string; step_name: string } }
  | { ToolCalled: { tool: string; args: Record<string, unknown> } }
  | { HumanApprovalRequired: { action: string } }
  | { StepCompleted: { result: Record<string, unknown>; latency_ms: number } }
  | { RunFinished: { cost_usd: number } };

/**
 * Subscribes to a live run SSE stream and accumulates events.
 *
 * Pass `null` to skip subscription (e.g., before a run is created).
 */
export function useAgentRun(runId: string | null = null) {
  const [events, setEvents] = useState<AgentEvent[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isFinished, setIsFinished] = useState(false);
  const clientRef = useRef<SseClient | null>(null);

  useEffect(() => {
    if (!runId) return;

    setEvents([]);
    setIsLoading(true);
    setIsFinished(false);

    const client = createSseClient<AgentEvent>(streamRunUrl(runId), {
      onEvent(event) {
        setEvents((prev) => [...prev, event]);
        if ("RunFinished" in event) {
          setIsFinished(true);
          setIsLoading(false);
          client.close();
        }
      },
      onError() {
        setIsLoading(false);
      },
      onClose() {
        setIsLoading(false);
      },
    });

    clientRef.current = client;
    return () => client.close();
  }, [runId]);

  return { events, isLoading, isFinished };
}

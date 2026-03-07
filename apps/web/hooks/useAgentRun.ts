"use client";

/**
 * Subscribes to a live run stream and returns placeholder state for the viewer.
 *
 * Replace this hook with a robust SSE client once the backend event contract is stable.
 */
export function useAgentRun() {
  return {
    events: [],
    isLoading: false,
  };
}

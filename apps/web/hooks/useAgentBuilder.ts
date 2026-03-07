"use client";

/**
 * Encapsulates natural-language builder state and placeholder actions.
 *
 * The final hook should coordinate streaming responses, optimistic UI state,
 * validation, and preview synchronization.
 */
export function useAgentBuilder() {
  return {
    input: "",
    isLoading: false,
  };
}

/**
 * Creates a placeholder EventSource-like contract description.
 *
 * A real implementation should wrap native `EventSource` with reconnection policy,
 * typed event parsing, and lifecycle cleanup.
 */
export function createSseClient(url: string) {
  return {
    url,
    connect() {
      return undefined;
    },
  };
}

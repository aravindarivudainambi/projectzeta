/**
 * Starts a new agent run with the given payload.
 *
 * Replace this placeholder with a typed POST helper once the execution API is implemented.
 */
export async function startRun(_payload: unknown) {
  return null;
}

/**
 * Returns a stream URL for a given run identifier.
 *
 * The final API client may return a richer subscription object instead of a plain string.
 */
export function streamRun(runId: string) {
  return `/runs/${runId}/stream`;
}

/**
 * Returns the frontend-visible API base URL.
 *
 * Centralize URL composition here so route handlers, hooks, and server components share one source.
 */
export function getApiBaseUrl() {
  return process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
}

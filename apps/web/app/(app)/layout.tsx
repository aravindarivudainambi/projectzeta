import type { ReactNode } from "react";

/**
 * Renders the authenticated application shell.
 *
 * Add sidebar navigation, top-level command surfaces, and session-aware providers here.
 */
export default function AppLayout({ children }: { children: ReactNode }) {
  return <section>{children}</section>;
}

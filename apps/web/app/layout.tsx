import type { ReactNode } from "react";
import "./globals.css";

/**
 * Renders the root document shell for the web application.
 *
 * This layout is the right place to add providers, fonts, theme wiring,
 * and any top-level telemetry boundaries once the implementation phase begins.
 */
export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

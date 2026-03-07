import type { ReactNode } from "react";
import "./globals.css";

export const metadata = {
  title: "Internal Agent Builder",
  description: "Prompt-driven agent authoring workspace.",
};

/**
 * Renders the root document shell for the web application.
 *
 * This layout is the right place to add providers, fonts, theme wiring,
 * and any top-level telemetry boundaries once the implementation phase begins.
 */
export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body className="bg-white text-zinc-950 antialiased">{children}</body>
    </html>
  );
}

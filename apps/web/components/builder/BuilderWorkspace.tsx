"use client";

import React, { useState } from "react";
import { useRouter } from "next/navigation";
import { NLBuilder } from "@/components/builder/NLBuilder";
import { WorkflowCanvas } from "@/components/builder/WorkflowCanvas/WorkflowCanvas";
import type { BuilderAgentConfig } from "@/lib/agent-config";

/**
 * Renders the prompt-driven agent authoring workspace used by the dashboard.
 *
 * Composes two views:
 *  - NL mode: natural-language streaming builder (NLBuilder)
 *  - Visual mode: interactive ReactFlow canvas (WorkflowCanvas)
 */
export function BuilderWorkspace() {
  const router = useRouter();
  const [viewMode, setViewMode] = useState<"nl" | "visual">("nl");
  const [parsedConfig, setParsedConfig] = useState<BuilderAgentConfig | null>(
    null,
  );

  return (
    <div className="flex h-[calc(100vh-3rem)] w-full flex-col overflow-hidden bg-zinc-50">
      <main className="mx-auto flex h-full w-full max-w-[1600px] min-w-[1280px] flex-1 flex-col overflow-hidden lg:flex-row">
        {viewMode === "visual" ? (
          <div className="relative h-full w-full flex-1">
            <WorkflowCanvas
              onReturn={() => setViewMode("nl")}
              initialConfig={parsedConfig}
            />
          </div>
        ) : (
          <NLBuilder
            onConfigReady={() => router.push("/agents")}
            onSwitchToCanvas={(config) => {
              setParsedConfig(config);
              setViewMode("visual");
            }}
          />
        )}
      </main>
    </div>
  );
}

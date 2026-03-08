"use client";

import { useState, useEffect } from "react";
import { ApprovalModal } from "./ApprovalModal";
import { StepTree } from "./StepTree";
import { LiveStatusBar, RunStatus } from "./LiveStatusBar";
import { CostTicker } from "./CostTicker";
import { RunTimeline, TimelineStep } from "./RunTimeline";
import { RunHeaderActions } from "./RunHeaderActions";
import { StepData } from "./StepCard";
import { useAgentRun, AgentEvent } from "@/hooks/useAgentRun";
import { useApproval } from "@/hooks/useApproval";

/**
 * Mock data for the demonstration when no real runId is provided.
 */
const MOCK_STEPS: StepData[] = [
  {
    id: "step_1",
    toolName: "google_search_gmail",
    status: "success",
    latencyMs: 340,
    inputArgs: { query: "label:important newer_than:7d" },
    outputResult: { total: 8, messages: ["msg-101", "msg-118"] },
  },
  {
    id: "step_2",
    toolName: "notion_create_page",
    status: "success",
    latencyMs: 850,
    inputArgs: { parent: { database_id: "db_feedback" }, properties: { Name: "Weekly Summary" } },
    outputResult: { id: "page_123", url: "https://notion.so/page_123" },
  },
  {
    id: "step_3",
    toolName: "google_create_calendar_event",
    status: "success",
    latencyMs: 210,
    inputArgs: {
      calendar_id: "primary",
      event: { summary: "Review weekly summary", start: { dateTime: "2026-03-07T15:00:00Z" } },
    },
    outputResult: { id: "evt_456", status: "confirmed" },
  },
  {
    id: "step_4",
    toolName: "human.request_approval",
    status: "paused",
    inputArgs: {
      action: "Restart production workers",
      riskContext: "May cause 10-20 seconds of downtime.",
    },
  },
];

function getToolError(result: Record<string, unknown>): string | undefined {
  const output = result.output;

  if (output && typeof output === "object" && !Array.isArray(output)) {
    const error = (output as { error?: unknown }).error;
    if (typeof error === "string" && error.trim()) {
      return error;
    }
  }

  if (typeof output === "string") {
    try {
      const parsed = JSON.parse(output) as { error?: unknown };
      if (typeof parsed.error === "string" && parsed.error.trim()) {
        return parsed.error;
      }
    } catch {
      return output;
    }
  }

  return undefined;
}

/** Transforms accumulated AgentEvent[] into the StepData[] the UI expects. */
function eventsToSteps(events: AgentEvent[]): StepData[] {
  const steps: StepData[] = [];
  let current: StepData | null = null;

  for (const event of events) {
    if ("StepStarted" in event) {
      if (current) steps.push(current);
      current = {
        id: event.StepStarted.step_id,
        toolName: event.StepStarted.step_name,
        status: "spinner",
      };
    } else if ("ToolCalled" in event && current) {
      current.toolName = event.ToolCalled.tool;
      current.inputArgs = event.ToolCalled.args;
    } else if ("StepCompleted" in event && current) {
      const rawResult = event.StepCompleted.result as Record<string, unknown>;
      const stepSucceeded = rawResult.success !== false;
      current.status = stepSucceeded ? "success" : "failed";
      current.latencyMs = event.StepCompleted.latency_ms;
      current.outputResult = rawResult;
      current.error = stepSucceeded ? undefined : getToolError(rawResult);
    } else if ("HumanApprovalRequired" in event && current) {
      current.status = "paused";
      current.inputArgs = { action: event.HumanApprovalRequired.action };
    }
  }

  if (current) steps.push(current);
  return steps;
}

interface LiveRunViewerProps {
  runId?: string;
}

export function LiveRunViewer({ runId: propRunId }: LiveRunViewerProps) {
  const isLive = !!propRunId;

  // Real backend hooks (only activate when a real runId is provided)
  const {
    events,
    isLoading: sseLoading,
    isFinished,
  } = useAgentRun(propRunId ?? null);
  const {
    isOpen: approvalIsOpen,
    approve,
    reject,
  } = useApproval(propRunId ?? null);

  // Derive live steps from SSE events
  const liveSteps = eventsToSteps(events);
  const liveFailedStep = liveSteps.find((step) => step.status === "failed");
  const liveCost = events.reduce((acc, e) => {
    if ("RunFinished" in e) return acc + e.RunFinished.cost_usd;
    return acc;
  }, 0);

  // ---- Mock simulation state (used when no real runId) ----
  const [mockRunId] = useState("run_prod_9Xq2pL");
  const [mockSteps, setMockSteps] = useState<StepData[]>(
    MOCK_STEPS.slice(0, 1),
  );
  const [mockRunStatus, setMockRunStatus] = useState<RunStatus>("running");
  const [mockCost, setMockCost] = useState(0.0012);
  const [showMockApproval, setShowMockApproval] = useState(false);

  useEffect(() => {
    if (isLive) return;

    let currentStepIdx = 1;
    const interval = setInterval(() => {
      if (currentStepIdx < MOCK_STEPS.length) {
        const nextStep = MOCK_STEPS[currentStepIdx];
        setMockSteps((prev) => [...prev, nextStep]);
        setMockCost((prev) => prev + 0.0031);
        if (nextStep.status === "paused") {
          setMockRunStatus("paused");
          setShowMockApproval(true);
          clearInterval(interval);
        }
        currentStepIdx++;
      }
    }, 1500);
    return () => clearInterval(interval);
  }, [isLive]);

  const handleMockApprove = () => {
    setShowMockApproval(false);
    setMockSteps((prev) => {
      const newSteps = [...prev];
      newSteps[newSteps.length - 1] = {
        ...newSteps[newSteps.length - 1],
        status: "spinner",
      };
      return newSteps;
    });
    setMockRunStatus("running");
    setTimeout(() => {
      setMockSteps((prev) => {
        const newSteps = [...prev];
        newSteps[newSteps.length - 1] = {
          ...newSteps[newSteps.length - 1],
          status: "success",
          latencyMs: 420,
          outputResult: { approved: true, approved_by: "user_req" },
        };
        return newSteps;
      });
      setMockRunStatus("completed");
    }, 1200);
  };

  const handleMockReject = () => {
    setShowMockApproval(false);
    setMockRunStatus("failed");
    setMockSteps((prev) => {
      const newSteps = [...prev];
      newSteps[newSteps.length - 1] = {
        ...newSteps[newSteps.length - 1],
        status: "failed",
        error: "Action rejected by human operator.",
      };
      return newSteps;
    });
  };

  // Pick live or mock data
  const displayRunId = isLive ? propRunId! : mockRunId;
  const displaySteps = isLive ? liveSteps : mockSteps;
  const displayCost = isLive ? liveCost : mockCost;
  const displayStatus: RunStatus = isLive
    ? liveFailedStep
      ? "failed"
      : isFinished
      ? "completed"
      : sseLoading
        ? "running"
        : "paused"
    : mockRunStatus;
  const displayTotalSteps = isLive ? displaySteps.length : MOCK_STEPS.length;
  const displayShowApproval = isLive ? approvalIsOpen : showMockApproval;

  const handleApprove = isLive ? approve : handleMockApprove;
  const handleReject = isLive ? reject : handleMockReject;

  const timelineSteps: TimelineStep[] = displaySteps.map((s) => ({
    id: s.id,
    name: s.toolName,
    status:
      s.status === "paused"
        ? "running"
        : s.status === "spinner"
          ? "running"
          : s.status,
    durationMs: s.latencyMs || 500,
  }));

  return (
    <div className="mx-auto flex h-full w-full max-w-6xl flex-col gap-6 bg-zinc-50 p-6 text-zinc-900">
      {/* Top Banner Area */}
      <header className="flex flex-col items-start justify-between gap-4 border-b border-zinc-200 pb-4 sm:flex-row sm:items-center">
        <div>
          <h1 className="flex items-center gap-2 font-mono text-xl font-bold text-zinc-950">
            Run Details
            <span className="rounded-full border border-zinc-200 bg-white px-2 py-0.5 font-sans text-xs tracking-wide text-zinc-500">
              {displayRunId}
            </span>
          </h1>
          <p className="mt-1 text-sm text-zinc-500">
            Live execution output from agent.
          </p>
        </div>

        <div className="flex items-center gap-4">
          <CostTicker cost={displayCost} />
          <div className="h-6 w-px bg-zinc-200"></div>
          <RunHeaderActions runId={displayRunId} />
        </div>
      </header>

      {/* Main Content Area */}
      <div className="grid flex-1 grid-cols-1 gap-6 lg:grid-cols-3">
        {/* Left Column: Flow & Activity */}
        <div className="flex flex-col gap-6 lg:col-span-2">
          <LiveStatusBar
            currentStep={displaySteps.length}
            totalSteps={displayTotalSteps}
            status={displayStatus}
            message={liveFailedStep?.error}
          />

          <div className="flex-1 overflow-y-auto rounded-xl border border-zinc-200 bg-white p-5 shadow-sm">
            <StepTree steps={displaySteps} />
          </div>
        </div>

        {/* Right Column: Meta & Observability */}
        <div className="flex flex-col gap-6">
          <div className="rounded-xl border border-zinc-200 bg-white p-5 shadow-sm">
            <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-zinc-700">
              Execution Timeline
            </h3>
            <RunTimeline steps={timelineSteps} totalDurationMs={3000} />
            <div className="mt-4 flex justify-between border-t border-zinc-200 pt-4 text-xs text-zinc-500">
              <span>Start</span>
              <span>Wait...</span>
            </div>
          </div>

          <div className="rounded-xl border border-zinc-200 bg-white p-5 shadow-sm">
            <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-zinc-700">
              Run Info
            </h3>
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-zinc-500">Agent</span>
                <span className="font-medium text-zinc-900">
                  Daily Report Bot
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-500">Triggered by</span>
                <span className="font-medium text-zinc-900">Cron Schedule</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-500">Total Tokens</span>
                <span className="font-medium text-zinc-900">
                  {(displayCost * 1850).toFixed(0)}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Interruption Modal */}
      <ApprovalModal
        isOpen={displayShowApproval}
        actionDescription="Execute production data write operation to master database."
        riskLevel="HIGH"
        contextData={{
          query:
            "UPDATE deployments SET status = 'restarting' WHERE target = 'workers'",
          impact: "Medium latency expected.",
        }}
        onApprove={handleApprove}
        onReject={handleReject}
      />
    </div>
  );
}

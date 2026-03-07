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
    toolName: "github.get_repository",
    status: "success",
    latencyMs: 340,
    inputArgs: { owner: "acme-corp", repo: "frontend-monorepo" },
    outputResult: { stars: 124, open_issues: 23, default_branch: "main" },
  },
  {
    id: "step_2",
    toolName: "jira.search_issues",
    status: "success",
    latencyMs: 850,
    inputArgs: { jql: "project = FRONTEND AND status = 'In Progress'" },
    outputResult: { total: 5, issues: ["FR-102", "FR-105", "FR-110"] },
  },
  {
    id: "step_3",
    toolName: "slack.send_message",
    status: "success",
    latencyMs: 210,
    inputArgs: {
      channel: "#engineering",
      text: "Daily status report generated.",
    },
    outputResult: { ok: true, message_ts: "12345678.90" },
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
      current.status = "success";
      current.latencyMs = event.StepCompleted.latency_ms;
      current.outputResult = event.StepCompleted.result;
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
    ? isFinished
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
    <div className="flex flex-col h-full bg-black text-slate-200 p-6 gap-6 w-full max-w-6xl mx-auto">
      {/* Top Banner Area */}
      <header className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center pb-2 border-b border-slate-800">
        <div>
          <h1 className="text-xl font-bold font-mono text-white flex items-center gap-2">
            Run Details
            <span className="bg-slate-800 text-slate-400 text-xs px-2 py-0.5 rounded-full font-sans tracking-wide">
              {displayRunId}
            </span>
          </h1>
          <p className="text-sm text-slate-500 mt-1">
            Live execution output from agent.
          </p>
        </div>

        <div className="flex items-center gap-4">
          <CostTicker cost={displayCost} />
          <div className="h-6 w-px bg-slate-800"></div>
          <RunHeaderActions runId={displayRunId} />
        </div>
      </header>

      {/* Main Content Area */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1">
        {/* Left Column: Flow & Activity */}
        <div className="lg:col-span-2 flex flex-col gap-6">
          <LiveStatusBar
            currentStep={displaySteps.length}
            totalSteps={displayTotalSteps}
            status={displayStatus}
          />

          <div className="flex-1 bg-slate-950 border border-slate-800 rounded-xl p-5 overflow-y-auto">
            <StepTree steps={displaySteps} />
          </div>
        </div>

        {/* Right Column: Meta & Observability */}
        <div className="flex flex-col gap-6">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5">
            <h3 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wider">
              Execution Timeline
            </h3>
            <RunTimeline steps={timelineSteps} totalDurationMs={3000} />
            <div className="mt-4 pt-4 border-t border-slate-800 flex justify-between text-xs text-slate-400">
              <span>Start</span>
              <span>Wait...</span>
            </div>
          </div>

          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5">
            <h3 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wider">
              Run Info
            </h3>
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-slate-500">Agent</span>
                <span className="text-slate-200 font-medium">
                  Daily Report Bot
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-500">Triggered by</span>
                <span className="text-slate-200 font-medium">
                  Cron Schedule
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-500">Total Tokens</span>
                <span className="text-slate-200 font-medium">
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

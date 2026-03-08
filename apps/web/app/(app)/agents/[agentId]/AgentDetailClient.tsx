"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { useRouter } from "next/navigation";
import {
  ArrowLeft,
  Play,
  Clock,
  CheckCircle2,
  XCircle,
  Loader2,
  Shield,
  Webhook,
  Bot,
  Calendar,
  Zap,
} from "lucide-react";
import {
  getAgent,
  listAgentRuns,
  startRun,
  streamRunUrl,
  approveRun,
  rejectRun,
} from "@api-client";
import type {
  AgentConfig,
  AgentEvent,
  RunHistoryEntry,
  RunStatus,
} from "@schema-types";

interface AgentDetailClientProps {
  agentId: string;
}

type StepProgress = {
  stepName: string;
  status: "pending" | "running" | "completed" | "failed" | "approval";
  toolCalled?: string;
  result?: unknown;
  latencyMs?: number;
};

function cronToHuman(cron: string): string {
  const parts = cron.split(" ");
  if (parts.length !== 5) return cron;
  const [minute, hour, , , dow] = parts;

  const dayNames: Record<string, string> = {
    "0": "Sunday",
    "1": "Monday",
    "2": "Tuesday",
    "3": "Wednesday",
    "4": "Thursday",
    "5": "Friday",
    "6": "Saturday",
    "7": "Sunday",
  };

  let schedule = "";
  if (dow !== "*") {
    schedule += `Every ${dayNames[dow] ?? `day ${dow}`}`;
  } else {
    schedule += "Daily";
  }

  if (hour !== "*" && minute !== "*") {
    const h = parseInt(hour, 10);
    const m = parseInt(minute, 10);
    const period = h >= 12 ? "PM" : "AM";
    const displayHour = h === 0 ? 12 : h > 12 ? h - 12 : h;
    schedule += ` at ${displayHour}:${m.toString().padStart(2, "0")} ${period}`;
  } else if (hour === "*") {
    schedule = "Every hour";
    if (minute !== "*" && minute !== "0") {
      schedule += ` at minute ${minute}`;
    }
  }

  return schedule;
}

function statusBadge(status: RunStatus) {
  switch (status) {
    case "Succeeded":
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700">
          <CheckCircle2 className="h-3 w-3" /> Succeeded
        </span>
      );
    case "Failed":
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-red-100 px-2 py-0.5 text-xs font-medium text-red-700">
          <XCircle className="h-3 w-3" /> Failed
        </span>
      );
    case "Running":
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-700">
          <Loader2 className="h-3 w-3 animate-spin" /> Running
        </span>
      );
    case "WaitingForApproval":
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-amber-100 px-2 py-0.5 text-xs font-medium text-amber-700">
          <Shield className="h-3 w-3" /> Awaiting Approval
        </span>
      );
    default:
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-600">
          <Clock className="h-3 w-3" /> {status}
        </span>
      );
  }
}

function timeAgo(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function stepResultFailed(result: unknown): boolean {
  return Boolean(
    result &&
      typeof result === "object" &&
      "success" in result &&
      (result as { success?: unknown }).success === false,
  );
}

export default function AgentDetailClient({ agentId }: AgentDetailClientProps) {
  const router = useRouter();
  const [agent, setAgent] = useState<AgentConfig | null>(null);
  const [runs, setRuns] = useState<RunHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [isRunning, setIsRunning] = useState(false);
  const [activeRunId, setActiveRunId] = useState<string | null>(null);
  const [stepProgress, setStepProgress] = useState<StepProgress[]>([]);
  const [runFinished, setRunFinished] = useState(false);
  const eventSourceRef = useRef<EventSource | null>(null);

  // Fetch agent and run history
  useEffect(() => {
    Promise.all([getAgent(agentId), listAgentRuns(agentId)])
      .then(([agentData, runsData]) => {
        setAgent(agentData);
        setRuns(runsData);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [agentId]);

  const handleRunNow = useCallback(async () => {
    if (!agent || isRunning) return;

    try {
      setIsRunning(true);
      setRunFinished(false);
      setStepProgress(
        agent.steps.map((s) => ({
          stepName: s.name,
          status: "pending" as const,
        })),
      );

      const result = await startRun({
        agent_id: agent.id,
        steps: agent.steps.map((s) => ({
          name: s.name,
          requires_approval: s.requires_approval ?? false,
          tool_name: s.tool_name,
        })),
      });

      setActiveRunId(result.run_id);

      // Open SSE stream
      const es = new EventSource(streamRunUrl(result.run_id));
      eventSourceRef.current = es;

      es.onmessage = (event) => {
        try {
          const parsed: AgentEvent = JSON.parse(event.data);

          if ("StepStarted" in parsed) {
            setStepProgress((prev) =>
              prev.map((s) =>
                s.stepName === parsed.StepStarted.step_name
                  ? { ...s, status: "running" as const }
                  : s,
              ),
            );
          } else if ("ToolCalled" in parsed) {
            setStepProgress((prev) => {
              const running = prev.find((s) => s.status === "running");
              if (!running) return prev;
              return prev.map((s) =>
                s.stepName === running.stepName
                  ? { ...s, toolCalled: parsed.ToolCalled.tool }
                  : s,
              );
            });
          } else if ("HumanApprovalRequired" in parsed) {
            setStepProgress((prev) => {
              const running = prev.find((s) => s.status === "running");
              if (!running) return prev;
              return prev.map((s) =>
                s.stepName === running.stepName
                  ? { ...s, status: "approval" as const }
                  : s,
              );
            });
          } else if ("StepCompleted" in parsed) {
            setStepProgress((prev) => {
              const running = prev.find(
                (s) => s.status === "running" || s.status === "approval",
              );
              if (!running) return prev;
              const failed = stepResultFailed(parsed.StepCompleted.result);
              return prev.map((s) =>
                s.stepName === running.stepName
                  ? {
                      ...s,
                      status: failed ? ("failed" as const) : ("completed" as const),
                      result: parsed.StepCompleted.result,
                      latencyMs: parsed.StepCompleted.latency_ms,
                    }
                  : s,
              );
            });
          } else if ("RunFinished" in parsed) {
            setRunFinished(true);
            setIsRunning(false);
            es.close();
            // Refresh run history
            listAgentRuns(agentId)
              .then(setRuns)
              .catch(() => {});
          }
        } catch {
          // ignore parse errors
        }
      };

      es.onerror = () => {
        es.close();
        setIsRunning(false);
      };
    } catch {
      setIsRunning(false);
    }
  }, [agent, isRunning, agentId]);

  const handleApprove = useCallback(async () => {
    if (!activeRunId) return;
    await approveRun(activeRunId);
  }, [activeRunId]);

  const handleReject = useCallback(async () => {
    if (!activeRunId) return;
    await rejectRun(activeRunId);
  }, [activeRunId]);

  // Cleanup EventSource on unmount
  useEffect(() => {
    return () => {
      eventSourceRef.current?.close();
    };
  }, []);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    );
  }

  if (!agent) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-4">
        <p className="text-gray-500">Agent not found.</p>
        <button
          onClick={() => router.push("/agents")}
          className="text-sm text-blue-600 hover:underline"
        >
          Back to agents
        </button>
      </div>
    );
  }

  const triggerLabel =
    agent.trigger === "Manual"
      ? "Manual"
      : typeof agent.trigger === "object" && "Schedule" in agent.trigger
        ? cronToHuman(agent.trigger.Schedule.cron)
        : typeof agent.trigger === "object" && "Event" in agent.trigger
          ? `${agent.trigger.Event.source}.${agent.trigger.Event.event}`
          : "Unknown";

  const triggerIcon =
    agent.trigger === "Manual" ? (
      <Play className="h-3.5 w-3.5" />
    ) : typeof agent.trigger === "object" && "Schedule" in agent.trigger ? (
      <Calendar className="h-3.5 w-3.5" />
    ) : (
      <Zap className="h-3.5 w-3.5" />
    );

  const toolNames = agent.steps
    .map((s) => s.tool_name)
    .filter(Boolean) as string[];

  return (
    <div className="mx-auto max-w-4xl space-y-8 p-6">
      {/* Back + Header */}
      <div>
        <button
          onClick={() => router.push("/agents")}
          className="mb-4 flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-700 transition-colors"
        >
          <ArrowLeft className="h-4 w-4" /> Back to Agents
        </button>

        <div className="flex items-start justify-between">
          <div className="flex items-center gap-4">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-indigo-100 text-indigo-600">
              <Bot className="h-6 w-6" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900">{agent.name}</h1>
              <div className="mt-1 flex items-center gap-2 text-sm text-gray-500">
                {triggerIcon}
                <span>{triggerLabel}</span>
                <span className="text-gray-300">|</span>
                <span>
                  {agent.steps.length} step
                  {agent.steps.length !== 1 ? "s" : ""}
                </span>
              </div>
            </div>
          </div>

          <button
            onClick={handleRunNow}
            disabled={isRunning}
            className="flex items-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm transition-colors hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isRunning ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Play className="h-4 w-4" fill="currentColor" />
            )}
            {isRunning ? "Running..." : "Run Now"}
          </button>
        </div>
      </div>

      {/* Tool Badges */}
      {toolNames.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {toolNames.map((tool) => (
            <span
              key={tool}
              className="flex items-center gap-1.5 rounded-md border border-gray-200 bg-gray-50 px-2.5 py-1 text-xs font-medium text-gray-600"
            >
              <Webhook className="h-3 w-3 text-indigo-500" />
              {tool}
            </span>
          ))}
        </div>
      )}

      {/* Steps */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-gray-900">
          Workflow Steps
        </h2>
        <ol className="relative ml-3 space-y-4 border-l border-gray-200">
          {agent.steps.map((step, idx) => {
            const progress = stepProgress[idx];
            const isActive = progress?.status === "running";
            const isComplete = progress?.status === "completed";
            const isFailed = progress?.status === "failed";
            const isApproval = progress?.status === "approval";

            return (
              <li key={step.id} className="relative pl-6">
                <span
                  className={`absolute -left-3 flex h-6 w-6 items-center justify-center rounded-full ring-4 ring-white text-xs font-bold border ${
                    isComplete
                      ? "bg-green-100 text-green-600 border-green-200"
                      : isActive
                        ? "bg-blue-100 text-blue-600 border-blue-200"
                        : isApproval
                          ? "bg-amber-100 text-amber-600 border-amber-200"
                          : isFailed
                            ? "bg-red-100 text-red-600 border-red-200"
                            : "bg-gray-100 text-gray-500 border-gray-200"
                  }`}
                >
                  {isComplete ? (
                    <CheckCircle2 className="h-3.5 w-3.5" />
                  ) : isActive ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : isApproval ? (
                    <Shield className="h-3.5 w-3.5" />
                  ) : isFailed ? (
                    <XCircle className="h-3.5 w-3.5" />
                  ) : (
                    idx + 1
                  )}
                </span>

                <div className="flex items-start justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-800">
                      {step.name}
                    </p>
                    <div className="mt-0.5 flex items-center gap-2 text-xs text-gray-500">
                      {step.tool_name && (
                        <span className="flex items-center gap-1">
                          <Webhook className="h-3 w-3" /> {step.tool_name}
                        </span>
                      )}
                      {step.requires_approval && (
                        <span className="flex items-center gap-1 text-amber-600">
                          <Shield className="h-3 w-3" /> Requires approval
                        </span>
                      )}
                    </div>
                    {progress?.toolCalled && isActive && (
                      <p className="mt-1 text-xs text-blue-600">
                        Calling {progress.toolCalled}...
                      </p>
                    )}
                    {progress?.latencyMs && isComplete && (
                      <p className="mt-1 text-xs text-gray-400">
                        {progress.latencyMs}ms
                      </p>
                    )}
                  </div>

                  {/* Approval buttons */}
                  {isApproval && (
                    <div className="flex gap-2">
                      <button
                        onClick={handleApprove}
                        className="rounded-md bg-green-600 px-3 py-1 text-xs font-medium text-white hover:bg-green-500 transition-colors"
                      >
                        Approve
                      </button>
                      <button
                        onClick={handleReject}
                        className="rounded-md bg-red-600 px-3 py-1 text-xs font-medium text-white hover:bg-red-500 transition-colors"
                      >
                        Reject
                      </button>
                    </div>
                  )}
                </div>
              </li>
            );
          })}
        </ol>

        {runFinished && (
          <div className="mt-4 rounded-lg border border-green-200 bg-green-50 p-3 text-sm text-green-700">
            Run completed successfully.
          </div>
        )}
      </section>

      {/* Run History */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-gray-900">
          Run History
        </h2>
        {runs.length === 0 ? (
          <p className="text-sm text-gray-500">
            No runs yet. Click &quot;Run Now&quot; to execute this agent.
          </p>
        ) : (
          <div className="space-y-3">
            {runs.map((run) => (
              <div
                key={run.run_id}
                className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    {statusBadge(run.status)}
                    <span className="text-xs text-gray-500">
                      {timeAgo(run.started_at)}
                    </span>
                    {run.finished_at && (
                      <span className="text-xs text-gray-400">
                        Duration:{" "}
                        {Math.round(
                          (new Date(run.finished_at).getTime() -
                            new Date(run.started_at).getTime()) /
                            1000,
                        )}
                        s
                      </span>
                    )}
                  </div>
                  <span className="font-mono text-xs text-gray-400">
                    {run.run_id.slice(0, 8)}
                  </span>
                </div>

                {run.step_results.length > 0 && (
                  <div className="mt-3 space-y-1.5 border-t border-gray-100 pt-3">
                    {run.step_results.map((step, i) => (
                      <div key={i} className="flex items-center gap-2 text-xs">
                        {step.success ? (
                          <CheckCircle2 className="h-3 w-3 text-green-500 shrink-0" />
                        ) : (
                          <XCircle className="h-3 w-3 text-red-500 shrink-0" />
                        )}
                        <span className="font-medium text-gray-700">
                          {step.step_name}
                        </span>
                        {step.tool_name && (
                          <span className="text-gray-400">
                            via {step.tool_name}
                          </span>
                        )}
                        <span className="ml-auto text-gray-400">
                          {step.latency_ms}ms
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

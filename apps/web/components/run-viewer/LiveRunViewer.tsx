"use client";

import { useState, useEffect } from "react";
import { ApprovalModal } from "./ApprovalModal";
import { StepTree } from "./StepTree";
import { LiveStatusBar, RunStatus } from "./LiveStatusBar";
import { CostTicker } from "./CostTicker";
import { RunTimeline, TimelineStep } from "./RunTimeline";
import { RunHeaderActions } from "./RunHeaderActions";
import { StepData } from "./StepCard";

/**
 * Mock data for the demonstration of Screen 3 UI.
 * In production, this would be driven by a useRunStream(runId) hook connecting to an SSE endpoint.
 */
const MOCK_STEPS: StepData[] = [
  {
    id: "step_1",
    toolName: "github.get_repository",
    status: "success",
    latencyMs: 340,
    inputArgs: { owner: "acme-corp", repo: "frontend-monorepo" },
    outputResult: { stars: 124, open_issues: 23, default_branch: "main" }
  },
  {
    id: "step_2",
    toolName: "jira.search_issues",
    status: "success",
    latencyMs: 850,
    inputArgs: { jql: "project = FRONTEND AND status = 'In Progress'" },
    outputResult: { total: 5, issues: ["FR-102", "FR-105", "FR-110"] }
  },
  {
    id: "step_3",
    toolName: "slack.send_message",
    status: "success",
    latencyMs: 210,
    inputArgs: { channel: "#engineering", text: "Daily status report generated." },
    outputResult: { ok: true, message_ts: "12345678.90" }
  },
  {
    id: "step_4",
    toolName: "human.request_approval",
    status: "paused",
    inputArgs: { 
      action: "Restart production workers", 
      riskContext: "May cause 10-20 seconds of downtime." 
    }
  }
];

export function LiveRunViewer() {
  const [runId] = useState("run_prod_9Xq2pL");
  const [steps, setSteps] = useState<StepData[]>(MOCK_STEPS.slice(0, 1));
  const [runStatus, setRunStatus] = useState<RunStatus>("running");
  const [cost, setCost] = useState(0.0012);
  const [showApproval, setShowApproval] = useState(false);

  // Simulation effect to show "Live" nature of the viewer
  useEffect(() => {
    let currentStepIdx = 1;
    
    const interval = setInterval(() => {
      if (currentStepIdx < MOCK_STEPS.length) {
        const nextStep = MOCK_STEPS[currentStepIdx];
        setSteps(prev => [...prev, nextStep]);
        setCost(prev => prev + 0.0031); // simulate cost increments
        
        if (nextStep.status === "paused") {
          setRunStatus("paused");
          setShowApproval(true);
          clearInterval(interval);
        }
        
        currentStepIdx++;
      }
    }, 1500);

    return () => clearInterval(interval);
  }, []);

  const handleApprove = () => {
    setShowApproval(false);
    
    // Convert paused step to spinner, then success
    setSteps(prev => {
      const newSteps = [...prev];
      newSteps[newSteps.length - 1] = { 
        ...newSteps[newSteps.length - 1], 
        status: "spinner" 
      };
      return newSteps;
    });
    setRunStatus("running");

    // Simulate approval processing completion
    setTimeout(() => {
      setSteps(prev => {
        const newSteps = [...prev];
        newSteps[newSteps.length - 1] = { 
          ...newSteps[newSteps.length - 1], 
          status: "success",
          latencyMs: 420,
          outputResult: { approved: true, approved_by: "user_req" }
        };
        return newSteps;
      });
      setRunStatus("completed");
    }, 1200);
  };

  const handleReject = () => {
    setShowApproval(false);
    setRunStatus("failed");
    setSteps(prev => {
      const newSteps = [...prev];
      newSteps[newSteps.length - 1] = { 
        ...newSteps[newSteps.length - 1], 
        status: "failed",
        error: "Action rejected by human operator."
      };
      return newSteps;
    });
  };

  // Convert rich steps array to minimal timeline format
  const timelineSteps: TimelineStep[] = steps.map(s => ({
    id: s.id,
    name: s.toolName,
    status: s.status === "paused" ? "running" : s.status === "spinner" ? "running" : s.status,
    durationMs: s.latencyMs || 500 // fallback duration for visualization
  }));

  return (
    <div className="flex flex-col h-full bg-black text-slate-200 p-6 gap-6 w-full max-w-6xl mx-auto">
      {/* Top Banner Area */}
      <header className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center pb-2 border-b border-slate-800">
        <div>
          <h1 className="text-xl font-bold font-mono text-white flex items-center gap-2">
            Run Details
            <span className="bg-slate-800 text-slate-400 text-xs px-2 py-0.5 rounded-full font-sans tracking-wide">
              {runId}
            </span>
          </h1>
          <p className="text-sm text-slate-500 mt-1">Live execution output from agent.</p>
        </div>
        
        <div className="flex items-center gap-4">
          <CostTicker cost={cost} />
          <div className="h-6 w-px bg-slate-800"></div>
          <RunHeaderActions runId={runId} />
        </div>
      </header>

      {/* Main Content Area */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1">
        {/* Left Column: Flow & Activity */}
        <div className="lg:col-span-2 flex flex-col gap-6">
          <LiveStatusBar 
            currentStep={steps.length} 
            totalSteps={MOCK_STEPS.length} 
            status={runStatus} 
          />
          
          <div className="flex-1 bg-slate-950 border border-slate-800 rounded-xl p-5 overflow-y-auto">
            <StepTree steps={steps} />
          </div>
        </div>

        {/* Right Column: Meta & Observability */}
        <div className="flex flex-col gap-6">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5">
            <h3 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wider">Execution Timeline</h3>
            <RunTimeline steps={timelineSteps} totalDurationMs={3000} />
            <div className="mt-4 pt-4 border-t border-slate-800 flex justify-between text-xs text-slate-400">
              <span>Start</span>
              <span>Wait...</span>
            </div>
          </div>
          
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5">
            <h3 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wider">Run Info</h3>
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-slate-500">Agent</span>
                <span className="text-slate-200 font-medium">Daily Report Bot</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-500">Triggered by</span>
                <span className="text-slate-200 font-medium">Cron Schedule</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-500">Total Tokens</span>
                <span className="text-slate-200 font-medium">{(cost * 1850).toFixed(0)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Interruption Modal */}
      <ApprovalModal 
        isOpen={showApproval}
        actionDescription="Execute production data write operation to master database."
        riskLevel="HIGH"
        contextData={{
          query: "UPDATE deployments SET status = 'restarting' WHERE target = 'workers'",
          impact: "Medium latency expected."
        }}
        onApprove={handleApprove}
        onReject={handleReject}
      />
    </div>
  );
}

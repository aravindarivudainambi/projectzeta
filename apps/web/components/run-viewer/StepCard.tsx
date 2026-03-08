"use client";

import { useState } from "react";
import {
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Clock,
  Loader2,
  XCircle,
  AlertCircle,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import clsx from "clsx";

export type StepStatus = "spinner" | "success" | "failed" | "paused";

export interface StepData {
  id: string;
  toolName: string;
  status: StepStatus;
  latencyMs?: number;
  inputArgs?: Record<string, any>;
  outputResult?: any;
  error?: string;
}

interface StepCardProps {
  step: StepData;
  isLast?: boolean;
}

export function StepCard({ step, isLast }: StepCardProps) {
  const [isInputOpen, setIsInputOpen] = useState(false);
  const [isOutputOpen, setIsOutputOpen] = useState(false);

  const StatusIcon = {
    spinner: <Loader2 className="w-5 h-5 text-blue-500 animate-spin" />,
    success: <CheckCircle2 className="w-5 h-5 text-emerald-500" />,
    failed: <XCircle className="w-5 h-5 text-red-500" />,
    paused: <AlertCircle className="w-5 h-5 text-amber-500" />,
  }[step.status];

  return (
    <div className="relative flex w-full">
      {/* Connector Line mapping from previous step */}
      {!isLast && (
        <div className="absolute bottom-0 left-[19px] top-10 z-0 -mb-10 w-[2px] bg-zinc-200" />
      )}

      {/* Node circle wrapper */}
      <div className="z-10 mt-3 flex w-10 flex-none justify-center bg-white">
        <div className="relative flex h-10 w-10 items-center justify-center rounded-full border border-zinc-200 bg-zinc-50 shadow-sm">
          {StatusIcon}
        </div>
      </div>

      <div className="flex-1 ml-4 py-2">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className={clsx(
            "overflow-hidden rounded-lg border bg-white p-4 shadow-sm transition-colors",
            step.status === "failed"
              ? "border-red-200 bg-red-50"
              : "border-zinc-200 hover:border-zinc-300",
          )}
        >
          {/* Header */}
          <div className="flex items-center justify-between mb-2">
            <h3 className="font-mono text-sm font-semibold text-zinc-900">
              {step.toolName}
            </h3>

            {step.latencyMs !== undefined && (
              <div className="flex items-center gap-1.5 rounded-md border border-zinc-200 bg-zinc-50 px-2 py-1 text-xs text-zinc-500">
                <Clock className="w-3.5 h-3.5" />
                <span>{step.latencyMs}ms</span>
              </div>
            )}
          </div>

          {/* Error Banner */}
          {step.status === "failed" && step.error && (
            <div className="mt-3 mb-3 rounded border border-red-200 bg-red-50 p-3 text-sm text-red-700">
              {step.error}
            </div>
          )}

          {/* I/O Sections */}
          <div className="flex flex-col gap-2 mt-4 space-y-1">
            {/* Inputs */}
            {step.inputArgs && (
              <div className="overflow-hidden rounded-md border border-zinc-200 bg-zinc-50">
                <button
                  onClick={() => setIsInputOpen(!isInputOpen)}
                  className="flex w-full items-center justify-between px-3 py-2 text-xs font-medium text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-800"
                >
                  <div className="flex items-center gap-2">
                    {isInputOpen ? (
                      <ChevronDown className="w-3.5 h-3.5" />
                    ) : (
                      <ChevronRight className="w-3.5 h-3.5" />
                    )}
                    <span>Input Arguments</span>
                  </div>
                  <span className="rounded bg-white px-1.5 py-0.5 font-mono text-[10px] text-zinc-500 ring-1 ring-zinc-200">
                    JSON
                  </span>
                </button>
                <AnimatePresence>
                  {isInputOpen && (
                    <motion.div
                      initial={{ height: 0 }}
                      animate={{ height: "auto" }}
                      exit={{ height: 0 }}
                      className="overflow-hidden border-t border-zinc-200"
                    >
                      <pre className="m-0 overflow-x-auto bg-white p-3 font-mono text-xs text-emerald-700">
                        {JSON.stringify(step.inputArgs, null, 2)}
                      </pre>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            )}

            {/* Outputs */}
            {step.outputResult && (
              <div className="overflow-hidden rounded-md border border-zinc-200 bg-zinc-50">
                <button
                  onClick={() => setIsOutputOpen(!isOutputOpen)}
                  className="flex w-full items-center justify-between px-3 py-2 text-xs font-medium text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-800"
                >
                  <div className="flex items-center gap-2">
                    {isOutputOpen ? (
                      <ChevronDown className="w-3.5 h-3.5" />
                    ) : (
                      <ChevronRight className="w-3.5 h-3.5" />
                    )}
                    <span>Output Result</span>
                  </div>
                  <span className="rounded bg-white px-1.5 py-0.5 font-mono text-[10px] text-zinc-500 ring-1 ring-zinc-200">
                    JSON
                  </span>
                </button>
                <AnimatePresence>
                  {isOutputOpen && (
                    <motion.div
                      initial={{ height: 0 }}
                      animate={{ height: "auto" }}
                      exit={{ height: 0 }}
                      className="overflow-hidden border-t border-zinc-200"
                    >
                      <pre className="m-0 overflow-x-auto bg-white p-3 font-mono text-xs text-blue-700">
                        {JSON.stringify(step.outputResult, null, 2)}
                      </pre>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            )}
          </div>
        </motion.div>
      </div>
    </div>
  );
}

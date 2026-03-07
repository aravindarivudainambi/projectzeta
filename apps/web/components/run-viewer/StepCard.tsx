"use client";

import { useState } from "react";
import { CheckCircle2, ChevronDown, ChevronRight, Clock, Loader2, XCircle, AlertCircle } from "lucide-react";
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
        <div className="absolute left-[19px] top-10 bottom-0 w-[2px] -mb-10 bg-slate-800 z-0" />
      )}

      {/* Node circle wrapper */}
      <div className="flex-none w-10 flex justify-center mt-3 z-10 bg-black">
        <div className="w-10 h-10 rounded-full border border-slate-800 bg-slate-900 flex items-center justify-center relative">
          {StatusIcon}
        </div>
      </div>

      <div className="flex-1 ml-4 py-2">
        <motion.div 
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className={clsx(
            "rounded-lg border bg-slate-900/50 p-4 shadow-sm backdrop-blur-sm transition-colors overflow-hidden",
            step.status === "failed" ? "border-red-500/50 bg-red-500/5" : "border-slate-800 hover:border-slate-700"
          )}
        >
          {/* Header */}
          <div className="flex items-center justify-between mb-2">
            <h3 className="font-mono text-sm font-semibold text-slate-200">
              {step.toolName}
            </h3>
            
            {step.latencyMs !== undefined && (
              <div className="flex items-center gap-1.5 text-xs text-slate-400 bg-slate-800/50 px-2 py-1 rounded-md border border-slate-700/50">
                <Clock className="w-3.5 h-3.5" />
                <span>{step.latencyMs}ms</span>
              </div>
            )}
          </div>

          {/* Error Banner */}
          {step.status === "failed" && step.error && (
            <div className="mt-3 mb-3 p-3 bg-red-500/10 border border-red-500/20 rounded text-sm text-red-400">
              {step.error}
            </div>
          )}

          {/* I/O Sections */}
          <div className="flex flex-col gap-2 mt-4 space-y-1">
            {/* Inputs */}
            {step.inputArgs && (
              <div className="rounded-md border border-slate-800 bg-slate-950/50 overflow-hidden">
                <button
                  onClick={() => setIsInputOpen(!isInputOpen)}
                  className="flex w-full items-center justify-between px-3 py-2 text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 transition-colors"
                >
                  <div className="flex items-center gap-2">
                    {isInputOpen ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
                    <span>Input Arguments</span>
                  </div>
                  <span className="font-mono text-[10px] bg-slate-800 px-1.5 py-0.5 rounded text-slate-500">JSON</span>
                </button>
                <AnimatePresence>
                  {isInputOpen && (
                    <motion.div
                      initial={{ height: 0 }}
                      animate={{ height: "auto" }}
                      exit={{ height: 0 }}
                      className="overflow-hidden border-t border-slate-800"
                    >
                      <pre className="p-3 text-xs font-mono text-emerald-400/80 overflow-x-auto m-0 bg-transparent">
                        {JSON.stringify(step.inputArgs, null, 2)}
                      </pre>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            )}

            {/* Outputs */}
            {step.outputResult && (
              <div className="rounded-md border border-slate-800 bg-slate-950/50 overflow-hidden">
                <button
                  onClick={() => setIsOutputOpen(!isOutputOpen)}
                  className="flex w-full items-center justify-between px-3 py-2 text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 transition-colors"
                >
                  <div className="flex items-center gap-2">
                    {isOutputOpen ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
                    <span>Output Result</span>
                  </div>
                  <span className="font-mono text-[10px] bg-slate-800 px-1.5 py-0.5 rounded text-slate-500">JSON</span>
                </button>
                <AnimatePresence>
                  {isOutputOpen && (
                    <motion.div
                      initial={{ height: 0 }}
                      animate={{ height: "auto" }}
                      exit={{ height: 0 }}
                      className="overflow-hidden border-t border-slate-800"
                    >
                      <pre className="p-3 text-xs font-mono text-blue-400/80 overflow-x-auto m-0 bg-transparent">
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

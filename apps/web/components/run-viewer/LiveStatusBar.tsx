"use client";

import { motion } from "framer-motion";
import { Activity, CheckCircle2, AlertCircle } from "lucide-react";
import clsx from "clsx";

export type RunStatus = "running" | "completed" | "failed" | "paused";

interface LiveStatusBarProps {
  currentStep: number;
  totalSteps: number;
  status: RunStatus;
  message?: string;
}

export function LiveStatusBar({ currentStep, totalSteps, status, message }: LiveStatusBarProps) {
  const statusColors = {
    running: "bg-blue-500/10 text-blue-500 border-blue-500/20",
    completed: "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
    failed: "bg-red-500/10 text-red-500 border-red-500/20",
    paused: "bg-amber-500/10 text-amber-500 border-amber-500/20",
  };

  const progress = totalSteps > 0 ? (currentStep / totalSteps) * 100 : 0;

  return (
    <div className={clsx("rounded-lg border px-4 py-3 flex flex-col gap-3", statusColors[status])}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 font-medium">
          {status === "running" && <Activity className="w-5 h-5 animate-pulse" />}
          {status === "completed" && <CheckCircle2 className="w-5 h-5" />}
          {status === "failed" && <AlertCircle className="w-5 h-5" />}
          {status === "paused" && <AlertCircle className="w-5 h-5" />}
          
          <span>
            {status === "running" && `Running step ${currentStep} of ${totalSteps}`}
            {status === "completed" && "Run completed successfully"}
            {status === "failed" && (message || "Run failed")}
            {status === "paused" && "Waiting for human approval"}
          </span>
        </div>
      </div>
      
      {/* Progress Bar */}
      <div className="h-2 w-full bg-white/10 rounded-full overflow-hidden relative">
        <motion.div 
          className="absolute inset-y-0 left-0 bg-current rounded-full"
          initial={{ width: 0 }}
          animate={{ width: `${progress}%` }}
          transition={{ duration: 0.5, ease: "easeInOut" }}
        />
      </div>
    </div>
  );
}

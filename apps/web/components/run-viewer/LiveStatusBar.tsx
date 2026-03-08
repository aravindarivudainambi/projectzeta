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

export function LiveStatusBar({
  currentStep,
  totalSteps,
  status,
  message,
}: LiveStatusBarProps) {
  const statusColors = {
    running: "border-blue-200 bg-blue-50 text-blue-700",
    completed: "border-emerald-200 bg-emerald-50 text-emerald-700",
    failed: "border-red-200 bg-red-50 text-red-700",
    paused: "border-amber-200 bg-amber-50 text-amber-700",
  };

  const progress = totalSteps > 0 ? (currentStep / totalSteps) * 100 : 0;

  return (
    <div
      className={clsx(
        "flex flex-col gap-3 rounded-lg border px-4 py-3 shadow-sm",
        statusColors[status],
      )}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 font-medium">
          {status === "running" && (
            <Activity className="w-5 h-5 animate-pulse" />
          )}
          {status === "completed" && <CheckCircle2 className="w-5 h-5" />}
          {status === "failed" && <AlertCircle className="w-5 h-5" />}
          {status === "paused" && <AlertCircle className="w-5 h-5" />}

          <span>
            {status === "running" &&
              `Running step ${currentStep} of ${totalSteps}`}
            {status === "completed" && "Run completed successfully"}
            {status === "failed" && (message || "Run failed")}
            {status === "paused" && "Waiting for human approval"}
          </span>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="relative h-2 w-full overflow-hidden rounded-full bg-white/80 ring-1 ring-black/5">
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

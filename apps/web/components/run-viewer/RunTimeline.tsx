"use client";

import { motion } from "framer-motion";
import clsx from "clsx";

export interface TimelineStep {
  id: string;
  name: string;
  durationMs: number;
  status: "success" | "failed" | "running";
}

interface RunTimelineProps {
  steps: TimelineStep[];
  totalDurationMs?: number;
}

export function RunTimeline({ steps, totalDurationMs }: RunTimelineProps) {
  // If no total duration is provided, calculate from steps
  const total =
    totalDurationMs || Math.max(...steps.map((s) => s.durationMs), 1);

  return (
    <div className="relative flex h-8 w-full items-center gap-[1px] overflow-hidden rounded-lg border border-zinc-200 bg-zinc-100 p-[1px]">
      {steps.map((step, index) => {
        // Minimum width of 2% so very fast steps are still visible
        const widthPercent = Math.max((step.durationMs / total) * 100, 2);

        return (
          <motion.div
            key={step.id}
            initial={{ opacity: 0, scaleX: 0 }}
            animate={{ opacity: 1, scaleX: 1 }}
            transition={{ duration: 0.3, delay: index * 0.1 }}
            className={clsx(
              "h-full origin-left relative group",
              step.status === "success" &&
                "bg-emerald-400 hover:bg-emerald-500",
              step.status === "failed" && "bg-red-400 hover:bg-red-500",
              step.status === "running" && "bg-blue-400 animate-pulse",
            )}
            style={{ width: `${widthPercent}%` }}
          >
            {/* Tooltip */}
            <div className="pointer-events-none absolute bottom-full left-1/2 z-10 mb-2 -translate-x-1/2 whitespace-nowrap rounded border border-zinc-200 bg-white px-2 py-1 text-xs opacity-0 shadow-sm transition-opacity group-hover:opacity-100">
              <span className="font-semibold text-zinc-900">{step.name}</span>
              <span className="ml-2 text-zinc-500">{step.durationMs}ms</span>
            </div>
          </motion.div>
        );
      })}
    </div>
  );
}

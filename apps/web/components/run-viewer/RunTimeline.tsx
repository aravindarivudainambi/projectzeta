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
  const total = totalDurationMs || Math.max(...steps.map(s => s.durationMs), 1);
  
  return (
    <div className="w-full flex items-center h-8 bg-slate-900/50 rounded-lg overflow-hidden border border-slate-800 relative gap-[1px] p-[1px]">
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
              step.status === "success" && "bg-emerald-500/40 hover:bg-emerald-500/60",
              step.status === "failed" && "bg-red-500/40 hover:bg-red-500/60",
              step.status === "running" && "bg-blue-500/40 animate-pulse"
            )}
            style={{ width: `${widthPercent}%` }}
          >
            {/* Tooltip */}
            <div className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 opacity-0 group-hover:opacity-100 transition-opacity bg-slate-800 text-xs px-2 py-1 rounded whitespace-nowrap z-10 pointer-events-none border border-slate-700">
              <span className="font-semibold text-slate-200">{step.name}</span>
              <span className="text-slate-400 ml-2">{step.durationMs}ms</span>
            </div>
          </motion.div>
        );
      })}
    </div>
  );
}

"use client";

import { AlertTriangle, Check, X, ShieldAlert } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import clsx from "clsx";

interface ApprovalModalProps {
  isOpen: boolean;
  actionDescription: string;
  riskLevel: "LOW" | "MEDIUM" | "HIGH";
  onApprove: () => void;
  onReject: () => void;
  contextData?: any;
}

export function ApprovalModal({
  isOpen,
  actionDescription,
  riskLevel,
  onApprove,
  onReject,
  contextData,
}: ApprovalModalProps) {
  if (!isOpen) return null;

  const riskStyles = {
    LOW: "text-blue-500 bg-blue-500/10 border-blue-500/20",
    MEDIUM: "text-amber-500 bg-amber-500/10 border-amber-500/20",
    HIGH: "text-red-500 bg-red-500/10 border-red-500/20",
  }[riskLevel];

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4 backdrop-blur-sm">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          className="flex w-full max-w-lg flex-col overflow-hidden rounded-xl border border-zinc-200 bg-white shadow-2xl"
          role="dialog"
          aria-modal="true"
        >
          {/* Header */}
          <div className="flex items-center gap-3 border-b border-zinc-200 px-6 py-4">
            <div className="rounded-full bg-amber-100 p-2 text-amber-600">
              <ShieldAlert className="w-6 h-6" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-zinc-950">
                Human Approval Required
              </h2>
              <p className="text-sm text-zinc-500">
                This action requires explicit sign-off to proceed.
              </p>
            </div>
          </div>

          {/* Content */}
          <div className="p-6 flex flex-col gap-4">
            <div className="flex items-start justify-between gap-4">
              <div className="font-medium text-zinc-900">
                {actionDescription}
              </div>
              <div
                className={clsx(
                  "px-2.5 py-1 rounded text-xs font-bold border whitespace-nowrap",
                  riskStyles,
                )}
              >
                {riskLevel} RISK
              </div>
            </div>

            {contextData && (
              <div className="mt-2 overflow-hidden rounded-md border border-zinc-200 bg-zinc-50 p-3">
                <div className="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-500">
                  Requested Tool Arguments
                </div>
                <pre className="m-0 overflow-x-auto bg-transparent font-mono text-xs text-emerald-700">
                  {JSON.stringify(contextData, null, 2)}
                </pre>
              </div>
            )}

            <div className="mt-2 flex items-center gap-2 text-sm text-zinc-500">
              <AlertTriangle className="w-4 h-4 text-amber-500" />
              <span>Approving this action cannot be undone.</span>
            </div>
          </div>

          {/* Footer controls */}
          <div className="flex items-center justify-end gap-3 border-t border-zinc-200 bg-zinc-50 px-6 py-4">
            <button
              onClick={onReject}
              className="flex items-center gap-2 rounded-lg px-4 py-2 font-medium text-zinc-600 transition-colors hover:bg-zinc-100 hover:text-zinc-900"
            >
              <X className="w-4 h-4" /> Reject
            </button>
            <button
              onClick={onApprove}
              className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-white bg-emerald-600 hover:bg-emerald-500 transition-colors shadow-sm"
            >
              <Check className="w-4 h-4" /> Approve Action
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}

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
  contextData
}: ApprovalModalProps) {
  if (!isOpen) return null;

  const riskStyles = {
    LOW: "text-blue-500 bg-blue-500/10 border-blue-500/20",
    MEDIUM: "text-amber-500 bg-amber-500/10 border-amber-500/20",
    HIGH: "text-red-500 bg-red-500/10 border-red-500/20",
  }[riskLevel];

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
        <motion.div 
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          className="bg-slate-900 border border-slate-700 w-full max-w-lg rounded-xl shadow-2xl overflow-hidden flex flex-col"
          role="dialog"
          aria-modal="true"
        >
          {/* Header */}
          <div className="px-6 py-4 border-b border-slate-800 flex items-center gap-3">
            <div className="p-2 bg-amber-500/10 rounded-full text-amber-500">
              <ShieldAlert className="w-6 h-6" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-slate-100">Human Approval Required</h2>
              <p className="text-sm text-slate-400">This action requires explicit sign-off to proceed.</p>
            </div>
          </div>

          {/* Content */}
          <div className="p-6 flex flex-col gap-4">
            <div className="flex items-start justify-between gap-4">
              <div className="text-slate-200 font-medium">
                {actionDescription}
              </div>
              <div className={clsx("px-2.5 py-1 rounded text-xs font-bold border whitespace-nowrap", riskStyles)}>
                {riskLevel} RISK
              </div>
            </div>

            {contextData && (
              <div className="bg-slate-950 border border-slate-800 rounded-md p-3 mt-2 overflow-hidden">
                <div className="text-xs font-semibold text-slate-500 mb-2 uppercase tracking-wider">Requested Tool Arguments</div>
                <pre className="text-xs font-mono text-emerald-400/80 overflow-x-auto m-0 bg-transparent">
                  {JSON.stringify(contextData, null, 2)}
                </pre>
              </div>
            )}
            
            <div className="flex items-center gap-2 mt-2 text-sm text-slate-400">
              <AlertTriangle className="w-4 h-4 text-amber-500" />
              <span>Approving this action cannot be undone.</span>
            </div>
          </div>

          {/* Footer controls */}
          <div className="px-6 py-4 border-t border-slate-800 bg-slate-900/50 flex items-center justify-end gap-3">
            <button 
              onClick={onReject}
              className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-slate-300 hover:text-white hover:bg-slate-800 transition-colors"
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

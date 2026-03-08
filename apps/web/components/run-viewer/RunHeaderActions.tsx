"use client";

import { Copy, Download, Check } from "lucide-react";
import { useState } from "react";

interface RunHeaderActionsProps {
  runId: string;
}

export function RunHeaderActions({ runId }: RunHeaderActionsProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(runId);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownload = () => {
    // In a real app this would download the JSON logs
    alert(`Downloading logs for run ${runId}`);
  };

  return (
    <div className="flex items-center gap-2">
      <button
        onClick={handleCopy}
        className="flex items-center gap-1.5 rounded-md border border-zinc-200 bg-white px-3 py-1.5 text-xs font-medium text-zinc-700 transition-colors hover:bg-zinc-50"
        title="Copy Run ID"
      >
        {copied ? (
          <Check className="w-3.5 h-3.5 text-emerald-500" />
        ) : (
          <Copy className="w-3.5 h-3.5" />
        )}
        <span className="hidden sm:inline">
          {copied ? "Copied" : "Copy ID"}
        </span>
      </button>

      <button
        onClick={handleDownload}
        className="flex items-center gap-1.5 rounded-md border border-zinc-200 bg-white px-3 py-1.5 text-xs font-medium text-zinc-700 transition-colors hover:bg-zinc-50"
        title="Download Logs"
      >
        <Download className="w-3.5 h-3.5" />
        <span className="hidden sm:inline">Export JSON</span>
      </button>
    </div>
  );
}

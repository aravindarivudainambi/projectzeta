"use client";

import React, { useState, useEffect } from "react";
import { Play, CheckCircle2, Webhook, Box, Bot } from "lucide-react";
import { cn } from "@/lib/utils";
import { motion, AnimatePresence } from "framer-motion";
import { extractDetectedTools, parseAgentConfig } from "@/lib/agent-config";

interface JsonPreviewProps {
  content: string;
  isStreaming: boolean;
  isValid: boolean;
  onSave: () => void;
  saveDisabled?: boolean;
  saveLabel?: string;
}

/**
 * Regex-based JSON syntax highlighter.
 *
 * Splits the raw string into tokens and wraps each in a colored span.
 * Handles partial JSON during streaming without crashing.
 */
function highlightJson(raw: string): React.ReactNode[] {
  if (!raw) return [];

  const nodes: React.ReactNode[] = [];
  // Match JSON tokens: strings (with key detection), numbers, booleans, null, structural chars
  const tokenRegex =
    /("(?:[^"\\]|\\.)*")\s*(?=:)|("(?:[^"\\]|\\.)*")|(-?\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b)|\b(true|false|null)\b|([{}[\]:,])/g;

  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = tokenRegex.exec(raw)) !== null) {
    // Emit any text before this match (whitespace, etc.)
    if (match.index > lastIndex) {
      nodes.push(
        <span key={key++} className="text-zinc-400">
          {raw.slice(lastIndex, match.index)}
        </span>,
      );
    }

    if (match[1] !== undefined) {
      // JSON key (string followed by colon)
      nodes.push(
        <span key={key++} className="text-indigo-400">
          {match[0]}
        </span>,
      );
    } else if (match[2] !== undefined) {
      // String value
      nodes.push(
        <span key={key++} className="text-emerald-400">
          {match[2]}
        </span>,
      );
    } else if (match[3] !== undefined) {
      // Number
      nodes.push(
        <span key={key++} className="text-amber-400">
          {match[3]}
        </span>,
      );
    } else if (match[4] !== undefined) {
      // Boolean or null
      nodes.push(
        <span key={key++} className="text-sky-400">
          {match[4]}
        </span>,
      );
    } else if (match[5] !== undefined) {
      // Structural character
      nodes.push(
        <span key={key++} className="text-zinc-500">
          {match[5]}
        </span>,
      );
    }

    lastIndex = match.index + match[0].length;
  }

  // Emit any trailing text
  if (lastIndex < raw.length) {
    nodes.push(
      <span key={key++} className="text-zinc-400">
        {raw.slice(lastIndex)}
      </span>,
    );
  }

  return nodes;
}

function formatJSON(jsonString: string): string {
  try {
    const parsed = JSON.parse(jsonString);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return jsonString;
  }
}

export function JsonPreview({
  content,
  isStreaming,
  isValid,
  onSave,
  saveDisabled,
  saveLabel = "Save & Deploy Agent",
}: JsonPreviewProps) {
  const [editableContent, setEditableContent] = useState("");
  const [isEditing, setIsEditing] = useState(false);

  useEffect(() => {
    if (!isEditing) {
      setEditableContent(formatJSON(content));
    }
  }, [content, isEditing]);

  const parsedContent = parseAgentConfig(editableContent);
  const isCurrentContentValid = parsedContent.success;
  const tools = parsedContent.success
    ? extractDetectedTools(parsedContent.data)
    : [];

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-xl border border-zinc-800 bg-zinc-950 font-mono text-sm text-zinc-300 shadow-2xl relative">
      {/* Header Bar */}
      <div className="flex items-center justify-between border-b border-zinc-800 bg-zinc-900 px-4 py-3 z-10">
        <div className="flex items-center gap-3">
          <Bot className="h-5 w-5 text-indigo-400" />
          <span className="text-xs font-semibold uppercase tracking-wider text-zinc-100">
            Agent Config
          </span>
          {isStreaming && (
            <span className="flex animate-pulse items-center gap-1.5 rounded-full bg-indigo-500/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-widest text-indigo-300">
              <span className="h-1.5 w-1.5 rounded-full bg-indigo-400" />
              Building
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          {!isStreaming && content && (
            <button
              onClick={() => setIsEditing(!isEditing)}
              className={cn(
                "rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors",
                isEditing
                  ? "border-zinc-700 bg-zinc-800 text-zinc-200"
                  : "border-transparent bg-transparent text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200",
              )}
            >
              {isEditing ? "Cancel Edit" : "Edit JSON"}
            </button>
          )}
        </div>
      </div>

      {/* Editor Content */}
      <div className="custom-scrollbar relative flex-1 overflow-auto p-5">
        {!content && !isStreaming ? (
          <div className="flex h-full w-full flex-col items-center justify-center text-zinc-600 opacity-60">
            <Box className="mb-4 h-12 w-12 opacity-50" />
            <p className="font-sans text-sm">
              Your agent configuration will stream here.
            </p>
          </div>
        ) : (
          <div className="relative">
            {isEditing ? (
              <textarea
                value={editableContent}
                onChange={(e) => setEditableContent(e.target.value)}
                className="h-full min-h-[400px] w-full resize-none bg-transparent font-mono leading-relaxed text-zinc-300 outline-none"
                spellCheck={false}
              />
            ) : (
              <pre className="whitespace-pre-wrap break-words leading-relaxed">
                {highlightJson(editableContent)}
                {isStreaming && (
                  <motion.span
                    animate={{ opacity: [1, 0, 1] }}
                    transition={{
                      duration: 0.8,
                      repeat: Infinity,
                      ease: "linear",
                    }}
                    className="ml-1 inline-block h-4 w-2.5 align-middle bg-indigo-400"
                  />
                )}
              </pre>
            )}
          </div>
        )}
      </div>

      {/* Bottom Action Bar */}
      <AnimatePresence>
        {!isStreaming && content && (
          <motion.div
            initial={{ y: 20, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 20, opacity: 0 }}
            className="z-10 flex flex-col gap-4 border-t border-zinc-800 bg-zinc-900 p-4"
          >
            {/* Detected Tools */}
            {tools.length > 0 && (
              <div className="flex items-center gap-2">
                <span className="font-sans text-xs text-zinc-500">
                  Detected Tools:
                </span>
                <div className="flex flex-wrap gap-2">
                  {tools.map((tool) => (
                    <span
                      key={tool}
                      className="flex items-center gap-1.5 rounded-md border border-zinc-700 bg-zinc-800 px-2 py-0.5 text-xs text-zinc-300"
                    >
                      <Webhook className="h-3 w-3 text-indigo-400" />
                      {tool}
                    </span>
                  ))}
                </div>
              </div>
            )}

            <div className="flex items-center justify-between border-t border-zinc-800 pt-2">
              <div className="flex items-center gap-2 text-xs">
                {isValid ? (
                  <span className="flex items-center gap-1.5 font-sans text-emerald-400">
                    <CheckCircle2 className="h-4 w-4" /> Valid Config
                  </span>
                ) : (
                  <span className="flex items-center gap-1.5 font-sans text-red-400">
                    <CheckCircle2 className="h-4 w-4 opacity-50" /> Needs Fixes
                  </span>
                )}
              </div>

              <button
                disabled={!isCurrentContentValid || saveDisabled}
                onClick={onSave}
                aria-label="Save agent configuration"
                className={cn(
                  "flex items-center gap-2 rounded-lg font-sans text-sm font-semibold transition-all px-4 py-2",
                  isCurrentContentValid && !saveDisabled
                    ? "bg-indigo-600 text-white shadow-lg shadow-indigo-500/20 hover:bg-indigo-500"
                    : "cursor-not-allowed bg-zinc-800 text-zinc-500",
                )}
              >
                <Play className="h-4 w-4" fill="currentColor" />
                {saveLabel}
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

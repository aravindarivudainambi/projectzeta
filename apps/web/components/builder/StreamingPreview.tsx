import React, { useState, useEffect } from 'react';
import { Play, CheckCircle2, Webhook, Box, Bot } from 'lucide-react';
import { cn } from '@/lib/utils';
import { motion, AnimatePresence } from 'framer-motion';

interface StreamingPreviewProps {
  content: string;
  isStreaming: boolean;
  onSave?: (content: string) => void;
  isValid?: boolean;
}

// Simple syntax highlighter for JSON
function formatJSON(jsonString: string) {
  try {
    const parsed = JSON.parse(jsonString);
    return JSON.stringify(parsed, null, 2);
  } catch (e) {
    return jsonString; // Keep raw string if not valid JSON yet
  }
}

// Simulate detecting tools dynamically from content
function extractTools(content: string): string[] {
  const tools = new Set<string>();
  if (content.toLowerCase().includes('slack')) tools.add('Slack');
  if (content.toLowerCase().includes('github')) tools.add('GitHub');
  if (content.toLowerCase().includes('jira')) tools.add('Jira');
  if (content.toLowerCase().includes('notion')) tools.add('Notion');
  if (content.toLowerCase().includes('salesforce')) tools.add('Salesforce');
  return Array.from(tools);
}

export function StreamingPreview({ content, isStreaming, onSave, isValid }: StreamingPreviewProps) {
  const [editableContent, setEditableContent] = useState('');
  const [isEditing, setIsEditing] = useState(false);

  useEffect(() => {
    if (!isEditing) {
      setEditableContent(formatJSON(content));
    }
  }, [content, isEditing]);

  const tools = extractTools(content);

  return (
    <div className="w-full h-full flex flex-col bg-zinc-950 text-zinc-300 rounded-xl overflow-hidden shadow-2xl border border-zinc-800 font-mono text-sm relative">
      {/* Header Bar */}
      <div className="flex items-center justify-between px-4 py-3 bg-zinc-900 border-b border-zinc-800 z-10">
        <div className="flex items-center gap-3">
          <Bot className="w-5 h-5 text-indigo-400" />
          <span className="font-semibold text-zinc-100 uppercase tracking-wider text-xs">
            Agent Config
          </span>
          {isStreaming && (
            <span className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 text-[10px] font-bold tracking-widest uppercase animate-pulse">
              <span className="w-1.5 h-1.5 rounded-full bg-indigo-400" />
              Building
            </span>
          )}
        </div>
        
        <div className="flex items-center gap-2">
          {!isStreaming && content && (
            <button
              onClick={() => setIsEditing(!isEditing)}
              className={cn(
                "px-3 py-1.5 text-xs font-medium rounded-lg transition-colors border",
                isEditing
                  ? "bg-zinc-800 border-zinc-700 text-zinc-200"
                  : "bg-transparent border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800"
              )}
            >
              {isEditing ? 'Cancel Edit' : 'Edit JSON'}
            </button>
          )}
        </div>
      </div>

      {/* Editor Content */}
      <div className="flex-1 relative overflow-auto p-5 custom-scrollbar">
        {!content && !isStreaming ? (
          <div className="w-full h-full flex flex-col items-center justify-center text-zinc-600 opacity-60">
            <Box className="w-12 h-12 mb-4 opacity-50" />
            <p className="font-sans text-sm">Your agent configuration will stream here.</p>
          </div>
        ) : (
          <div className="relative">
            {isEditing ? (
              <textarea
                value={editableContent}
                onChange={(e) => setEditableContent(e.target.value)}
                className="w-full h-full min-h-[400px] bg-transparent resize-none outline-none text-zinc-300 leading-relaxed font-mono"
                spellCheck={false}
              />
            ) : (
              <pre className="whitespace-pre-wrap break-words leading-relaxed">
                {editableContent}
                {isStreaming && (
                  <motion.span
                    animate={{ opacity: [1, 0, 1] }}
                    transition={{ duration: 0.8, repeat: Infinity, ease: 'linear' }}
                    className="inline-block w-2.5 h-4 ml-1 align-middle bg-indigo-400"
                  />
                )}
              </pre>
            )}
          </div>
        )}
      </div>

      {/* Bottom Action Bar */}
      <AnimatePresence>
        {(!isStreaming && content) && (
          <motion.div
            initial={{ y: 20, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 20, opacity: 0 }}
            className="p-4 bg-zinc-900 border-t border-zinc-800 flex flex-col gap-4 z-10"
          >
            {/* Detected Tools */}
            {tools.length > 0 && (
              <div className="flex items-center gap-2">
                <span className="text-xs text-zinc-500 font-sans">Detected Tools:</span>
                <div className="flex flex-wrap gap-2">
                  {tools.map(tool => (
                    <span
                      key={tool}
                      className="px-2 py-0.5 rounded-md bg-zinc-800 border border-zinc-700 text-zinc-300 text-xs flex items-center gap-1.5"
                    >
                      <Webhook className="w-3 h-3 text-indigo-400" />
                      {tool}
                    </span>
                  ))}
                </div>
              </div>
            )}

            <div className="flex items-center justify-between pt-2 border-t border-zinc-800">
              <div className="flex items-center gap-2 text-xs">
                {isValid ? (
                  <span className="text-emerald-400 flex items-center gap-1.5 font-sans">
                    <CheckCircle2 className="w-4 h-4" /> Valid Schema
                  </span>
                ) : (
                  <span className="text-red-400 flex items-center gap-1.5 font-sans">
                    <CheckCircle2 className="w-4 h-4 opacity-50" /> Parsing Error
                  </span>
                )}
              </div>
              
              <button
                disabled={!isValid}
                onClick={() => onSave?.(editableContent)}
                className={cn(
                  "px-4 py-2 rounded-lg font-sans text-sm font-semibold transition-all flex items-center gap-2",
                  isValid
                    ? "bg-indigo-600 hover:bg-indigo-500 text-white shadow-lg shadow-indigo-500/20"
                    : "bg-zinc-800 text-zinc-500 cursor-not-allowed"
                )}
              >
                <Play className="w-4 h-4" fill="currentColor" />
                Save &&nbsp;Deploy Agent
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

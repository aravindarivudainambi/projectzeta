import React, { useRef, useEffect } from 'react';
import { Send, Sparkles } from 'lucide-react';
import { cn } from '@/lib/utils';
import { motion } from 'framer-motion';

interface PromptInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: (promptOverride?: string) => void;
  isStreaming: boolean;
  hasError?: boolean;
}

const EXAMPLES = [
  "Summarize Gmail updates into a Notion page every Friday",
  "Create a Google Calendar event when a Notion task is approved",
  "Search Google Drive for onboarding docs and save the summary to Notion"
];

export function PromptInput({ value, onChange, onSubmit, isStreaming, hasError }: PromptInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.max(64, textareaRef.current.scrollHeight)}px`;
    }
  }, [value]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (!isStreaming) onSubmit();
    }
  };

  const handleExampleClick = (example: string) => {
    if (isStreaming) return;
    onChange(example);
    onSubmit(example);
  };

  return (
    <div className="w-full max-w-3xl mx-auto flex flex-col gap-4">
      <div className="relative group">
        <motion.div
          animate={hasError ? { x: [-10, 10, -10, 10, -5, 5, 0] } : {}}
          transition={{ duration: 0.4 }}
          className={cn(
            "relative rounded-2xl border bg-white shadow-sm overflow-hidden transition-all duration-200",
            hasError ? "border-red-500 shadow-red-500/10" : "border-zinc-200 focus-within:border-zinc-300 focus-within:shadow-md focus-within:ring-4 focus-within:ring-zinc-100"
          )}
        >
          <textarea
            ref={textareaRef}
            value={value}
            onChange={(event) => onChange(event.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Describe your workflow in plain English..."
            className="w-full resize-none bg-transparent px-5 py-5 pr-14 outline-none min-h-[64px] max-h-[400px] text-zinc-900 placeholder:text-zinc-400 placeholder:font-light"
            disabled={isStreaming}
          />
          
          <div className="absolute right-3 bottom-3 flex items-center justify-center">
            <button
              type="button"
              onClick={() => onSubmit()}
              disabled={!(value || '').trim() || isStreaming}
              aria-label="Generate agent configuration"
              className={cn(
                "p-2 rounded-xl flex items-center justify-center transition-all duration-200",
                (value || '').trim() && !isStreaming
                  ? "bg-zinc-900 text-white hover:bg-zinc-800 shadow-sm"
                  : "bg-zinc-100 text-zinc-400 cursor-not-allowed"
              )}
            >
              <Send className="w-4 h-4" />
            </button>
          </div>
        </motion.div>
      </div>

      <div className="flex flex-wrap items-center gap-2 px-1">
        <span className="text-xs font-medium text-zinc-400 uppercase tracking-wider flex items-center gap-1.5 ml-1 mr-2">
          <Sparkles className="w-3.5 h-3.5" />
          Examples
        </span>
        {EXAMPLES.map((example) => (
          <button
            key={example}
            type="button"
            onClick={() => handleExampleClick(example)}
            disabled={isStreaming}
            className="px-3 py-1.5 rounded-full border border-zinc-200 bg-white text-xs font-medium text-zinc-600 hover:border-zinc-300 hover:bg-zinc-50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {example}
          </button>
        ))}
      </div>
    </div>
  );
}

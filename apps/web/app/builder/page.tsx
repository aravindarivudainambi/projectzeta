'use client';

import React, { useState, useEffect } from 'react';
import { useChat } from '@ai-sdk/react';
import { LayoutGrid } from 'lucide-react';

import { PromptInput } from '@/components/builder/PromptInput';
import { StreamingPreview } from '@/components/builder/StreamingPreview';
import { StatusBanners, StatusBannerType } from '@/components/builder/StatusBanners';
import { WorkflowCanvas } from '@/components/builder/WorkflowCanvas/WorkflowCanvas';

export default function AgentBuilderPage() {
  const { messages, input, handleInputChange, handleSubmit, setMessages, isLoading, error } = useChat({
    api: '/api/agent/build',
  });

  const [viewMode, setViewMode] = useState<'nl' | 'visual'>('nl');

  const [bannerState, setBannerState] = useState<{
    visible: boolean;
    type: StatusBannerType;
    title: string;
    message?: string;
  }>({ visible: false, type: 'info', title: '' });

  const [promptError, setPromptError] = useState(false);
  const [isValidJson, setIsValidJson] = useState(false);

  // The latest stream content arrives in the last message if role is assistant
  const latestMessage = messages.length > 0 ? messages[messages.length - 1] : null;
  const content = latestMessage?.role === 'assistant' ? latestMessage.content : '';

  // Handle stream timeout scenario (10s)
  useEffect(() => {
    let timeoutId: NodeJS.Timeout;
    if (isLoading && !content) {
      timeoutId = setTimeout(() => {
        setBannerState({
          visible: true,
          type: 'timeout',
          title: 'Taking longer than usual...',
          message: 'The LLM API is responding slowly.',
        });
      }, 5000);
    } else if (content || !isLoading) {
      setBannerState(prev => prev.type === 'timeout' ? { ...prev, visible: false } : prev);
    }
    return () => clearTimeout(timeoutId);
  }, [isLoading, content]);

  // Handle schema validation and error handling after completion
  useEffect(() => {
    if (!isLoading && content) {
      try {
        const parsed = JSON.parse(content);
        setIsValidJson(true);
        setBannerState({ visible: false, type: 'info', title: '' }); // Clear errors
        
        // Check for tools
        const hasTools = ('steps' in parsed && Array.isArray(parsed.steps) && parsed.steps.length > 0) || content.toLowerCase().includes('github') || content.toLowerCase().includes('slack');
        
        if (!hasTools) {
          setBannerState({
            visible: true,
            type: 'warning',
            title: 'No integrations detected',
            message: 'Is your workflow missing a tool?',
          });
        }
      } catch (e) {
        setIsValidJson(false);
        setBannerState({
          visible: true,
          type: 'error',
          title: "Config couldn't be parsed",
          message: "The AI returned an invalid format. Try rephrasing your prompt.",
        });
      }
    } else if (isLoading) {
      setIsValidJson(false);
    }
  }, [isLoading, content]);

  // Handle Form Submit
  const onFormSubmit = (e?: React.FormEvent<HTMLFormElement>) => {
    if (e) e.preventDefault();
    if (!input.trim()) {
      setPromptError(true);
      setTimeout(() => setPromptError(false), 1000); // Reset shake animation
      return;
    }
    
    // Clear previous UI state and start stream
    setBannerState({ visible: false, type: 'info', title: '' });
    setMessages([]);
    handleSubmit();
  };

  const handleManualSave = (finalContent: string) => {
    alert("Saved Agent Configuration:\\n" + finalContent);
  };

  return (
    <div className="h-screen w-full bg-zinc-50 flex flex-col items-center overflow-hidden">
      {/* Top Navigation / Header */}
      <header className="w-full h-14 border-b bg-white flex items-center justify-between px-6 shrink-0 z-50">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 bg-zinc-900 rounded-md flex items-center justify-center">
            <LayoutGrid className="w-3 h-3 text-white" />
          </div>
          <span className="font-semibold text-sm">Internal Agent Builder</span>
        </div>
        <div className="flex gap-3">
          <button className="text-xs font-medium text-zinc-500 hover:text-zinc-900 transition-colors">Documentation</button>
        </div>
      </header>

      {/* Main Workspace */}
      <main className="flex-1 w-full flex flex-col lg:flex-row overflow-hidden max-w-[1600px] mx-auto min-w-[1280px]">
        {viewMode === 'visual' ? (
          <div className="w-full h-full flex-1 relative">
            <WorkflowCanvas onReturn={() => setViewMode('nl')} />
          </div>
        ) : (
          <>
            {/* Left Side: Input Pane */}
            <section className="flex-1 p-8 lg:p-12 overflow-y-auto flex flex-col">
              <div className="max-w-2xl mx-auto w-full flex flex-col pt-[10vh]">
                <h1 className="text-3xl font-semibold tracking-tight text-zinc-900 mb-2">
                  Create a new Agent
                </h1>
                <p className="text-zinc-500 mb-10 text-sm">
                  Describe your workflow in plain English. We'll automatically wire the integrations and logic.
                </p>

                {/* Error / Status Displays */}
                <div className="mb-6 z-20">
                  <StatusBanners
                    {...bannerState}
                    action={bannerState.type === 'error' || bannerState.type === 'timeout' ? {
                      label: 'Retry',
                      onClick: onFormSubmit
                    } : undefined}
                  />
                </div>

                {/* Prompt Form */}
                <form onSubmit={onFormSubmit}>
                  <PromptInput
                    value={input}
                    onChange={handleInputChange}
                    onSubmit={onFormSubmit}
                    isStreaming={isLoading}
                    hasError={promptError}
                  />
                </form>
              </div>
            </section>

            {/* Right Side: Preview Pane */}
            <section className="w-full lg:w-[45%] xl:w-[50%] p-6 bg-zinc-100 flex items-center justify-center shrink-0 border-l relative shadow-inner">
              <div className="absolute top-6 right-6 z-20">
                 <button 
                  onClick={() => setViewMode('visual')}
                  className="px-3 py-1.5 rounded-lg border bg-white shadow-sm text-xs font-medium text-zinc-600 hover:bg-zinc-50 flex items-center gap-2 transition-colors"
                 >
                   <LayoutGrid className="w-3.5 h-3.5" />
                   Visual Canvas
                 </button>
              </div>
              <div className="w-full h-[85vh] max-h-[900px]">
                <StreamingPreview
                  content={content}
                  isStreaming={isLoading}
                  isValid={isValidJson}
                  onSave={handleManualSave}
                />
              </div>
            </section>
          </>
        )}
      </main>
    </div>
  );
}

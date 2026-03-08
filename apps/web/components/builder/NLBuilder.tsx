"use client";

import React, { useState, useEffect } from "react";
import { useCompletion } from "@ai-sdk/react";
import { LayoutGrid } from "lucide-react";

import { PromptInput } from "@/components/builder/PromptInput";
import { JsonPreview } from "@/components/builder/JsonPreview";
import {
  StatusBanners,
  StatusBannerType,
} from "@/components/builder/StatusBanners";
import {
  BuilderAgentConfig,
  extractDetectedTools,
  parseAgentConfig,
} from "@/lib/agent-config";
import { createAgent } from "@api-client";
import type { AgentConfig } from "@schema-types";

interface NLBuilderProps {
  onConfigReady: (config: AgentConfig) => void;
  onSwitchToCanvas: (config: BuilderAgentConfig | null) => void;
}

/**
 * Natural-language agent builder powered by Vercel AI SDK useCompletion.
 *
 * Streams JSON config from /api/agent/build token-by-token,
 * validates in real time, and persists via createAgent on save.
 */
export function NLBuilder({ onConfigReady, onSwitchToCanvas }: NLBuilderProps) {
  const {
    completion,
    input,
    setInput,
    handleSubmit,
    isLoading,
    stop,
    complete,
  } = useCompletion({
    api: "/api/agent/build",
    streamProtocol: "text",
  });

  const [promptError, setPromptError] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [lastPrompt, setLastPrompt] = useState("");
  const [bannerState, setBannerState] = useState<{
    visible: boolean;
    type: StatusBannerType;
    title: string;
    message?: string;
  }>({ visible: false, type: "info", title: "" });

  // The streamed content is the completion
  const streamedContent = completion;

  // Parse the streamed content
  const parseResult = parseAgentConfig(streamedContent);
  const parsedConfig = parseResult.success ? parseResult.data : null;
  const isValidJson = parseResult.success;

  // Timeout banner when streaming takes > 5s
  useEffect(() => {
    if (!isLoading) return;

    const timeoutId = window.setTimeout(() => {
      setBannerState({
        visible: true,
        type: "timeout",
        title: "Taking longer than usual...",
        message:
          "The agent is still being generated. You can keep waiting or cancel the stream.",
      });
    }, 5000);

    return () => window.clearTimeout(timeoutId);
  }, [isLoading]);

  // Update banners when streaming completes
  useEffect(() => {
    if (isLoading || !streamedContent) return;

    const result = parseAgentConfig(streamedContent);
    if (result.success) {
      const detectedTools = extractDetectedTools(result.data);
      if (detectedTools.length === 0) {
        setBannerState({
          visible: true,
          type: "warning",
          title: "No integrations detected",
          message: "Is your workflow missing a tool?",
        });
      } else {
        setBannerState({ visible: false, type: "info", title: "" });
      }
    } else {
      setBannerState({
        visible: true,
        type: "error",
        title: "Config couldn't be parsed",
        message:
          "Try rephrasing your prompt or editing the generated JSON before saving.",
      });
    }
  }, [isLoading, streamedContent]);

  // Clear banner when a new stream begins
  useEffect(() => {
    if (isLoading) {
      setBannerState({ visible: false, type: "info", title: "" });
    }
  }, [isLoading]);

  const onFormSubmit = (promptOverride?: string) => {
    const value = (promptOverride ?? input).trim();
    if (!value || isLoading) {
      setPromptError(true);
      window.setTimeout(() => setPromptError(false), 1000);
      return;
    }

    setLastPrompt(value);

    if (promptOverride) {
      // Example pill click — use complete() to send directly
      setInput(promptOverride);
      complete(promptOverride);
    } else {
      // Normal form submit — useCompletion reads from input state
      handleSubmit();
    }
  };

  const handleSave = async () => {
    if (!parsedConfig) return;

    try {
      setIsSaving(true);
      const saved = await createAgent({
        name: parsedConfig.name,
        trigger: parsedConfig.trigger,
        steps: parsedConfig.steps.map((s) => ({
          name: s.name,
          requires_approval: Boolean(s.requires_approval),
          ...(s.tool_name ? { tool_name: s.tool_name } : {}),
        })),
      });

      setBannerState({
        visible: true,
        type: "info",
        title: "Agent saved",
        message: `${saved.name} (${saved.id.slice(0, 8)}...) is now available in your agents list.`,
      });

      onConfigReady(saved);
    } catch (error) {
      setBannerState({
        visible: true,
        type: "error",
        title: "Failed to save agent",
        message:
          error instanceof Error
            ? error.message
            : "Check your connection and try again.",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleCancelGeneration = () => {
    stop();
    setBannerState({
      visible: true,
      type: "info",
      title: "Generation cancelled",
      message: "You can edit the prompt and try again.",
    });
  };

  return (
    <>
      <section className="flex flex-1 flex-col overflow-y-auto p-8 lg:p-12">
        <div className="mx-auto flex w-full max-w-2xl flex-col pt-[10vh]">
          <div className="mb-8 flex items-start gap-3">
            <div className="mt-1 flex h-9 w-9 items-center justify-center rounded-xl bg-zinc-900">
              <LayoutGrid className="h-4 w-4 text-white" />
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-[0.28em] text-zinc-500">
                Dashboard
              </p>
              <h1 className="mt-2 text-3xl font-semibold tracking-tight text-zinc-900">
                Create a new Agent
              </h1>
              <p className="mt-2 text-sm text-zinc-500">
                Describe your workflow in plain English. We&apos;ll
                automatically wire the integrations and logic.
              </p>
            </div>
          </div>

          <div className="z-20 mb-6">
            <StatusBanners
              {...bannerState}
              action={
                bannerState.type === "error" || bannerState.type === "timeout"
                  ? {
                      label:
                        bannerState.type === "timeout" ? "Cancel" : "Retry",
                      onClick:
                        bannerState.type === "timeout"
                          ? handleCancelGeneration
                          : () => {
                              if (lastPrompt) {
                                onFormSubmit(lastPrompt);
                              }
                            },
                    }
                  : undefined
              }
            />
          </div>

          <form
            onSubmit={(event) => {
              event.preventDefault();
              onFormSubmit();
            }}
          >
            <PromptInput
              value={input}
              onChange={setInput}
              onSubmit={onFormSubmit}
              isStreaming={isLoading}
              hasError={promptError}
            />
          </form>
        </div>
      </section>

      <section className="relative flex w-full shrink-0 items-center justify-center border-l bg-zinc-100 p-6 shadow-inner lg:w-[45%] xl:w-[50%]">
        <div className="absolute right-6 top-6 z-20">
          <button
            type="button"
            onClick={() => onSwitchToCanvas(parsedConfig)}
            className="flex items-center gap-2 rounded-lg border bg-white px-3 py-1.5 text-xs font-medium text-zinc-600 shadow-sm transition-colors hover:bg-zinc-50"
          >
            <LayoutGrid className="h-3.5 w-3.5" />
            {parsedConfig ? "Open in Canvas" : "Visual Canvas"}
          </button>
        </div>
        <div className="h-[85vh] max-h-[900px] w-full">
          <JsonPreview
            content={streamedContent}
            isStreaming={isLoading}
            isValid={isValidJson}
            onSave={handleSave}
            saveDisabled={isSaving}
            saveLabel={isSaving ? "Saving..." : "Save & Deploy Agent"}
          />
        </div>
      </section>
    </>
  );
}

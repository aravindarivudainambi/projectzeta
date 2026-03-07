"use client";

import React, { useEffect, useRef, useState } from "react";
import { LayoutGrid } from "lucide-react";

import { PromptInput } from "@/components/builder/PromptInput";
import { StreamingPreview } from "@/components/builder/StreamingPreview";
import {
  StatusBanners,
  StatusBannerType,
} from "@/components/builder/StatusBanners";
import { WorkflowCanvas } from "@/components/builder/WorkflowCanvas/WorkflowCanvas";
import {
  BuilderAgentConfig,
  extractDetectedTools,
  parseAgentConfig,
} from "@/lib/agent-config";

/**
 * Renders the prompt-driven agent authoring workspace used by the dashboard.
 */
export function BuilderWorkspace() {
  const abortControllerRef = useRef<AbortController | null>(null);

  const [viewMode, setViewMode] = useState<"nl" | "visual">("nl");

  const [input, setInput] = useState("");
  const [content, setContent] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [lastPrompt, setLastPrompt] = useState("");
  const [parsedConfig, setParsedConfig] = useState<BuilderAgentConfig | null>(
    null,
  );
  const [promptError, setPromptError] = useState(false);
  const [isValidJson, setIsValidJson] = useState(false);
  const [bannerState, setBannerState] = useState<{
    visible: boolean;
    type: StatusBannerType;
    title: string;
    message?: string;
  }>({ visible: false, type: "info", title: "" });

  useEffect(() => {
    if (!isLoading) {
      return;
    }

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

  useEffect(() => {
    if (!content) {
      setParsedConfig(null);
      setIsValidJson(false);
      return;
    }

    const result = parseAgentConfig(content);

    if (result.success) {
      setParsedConfig(result.data);
      setIsValidJson(true);

      if (!isLoading) {
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
      }
    } else {
      setParsedConfig(null);
      setIsValidJson(false);

      if (!isLoading) {
        setBannerState({
          visible: true,
          type: "error",
          title: "Config couldn't be parsed",
          message:
            "Try rephrasing your prompt or editing the generated JSON before saving.",
        });
      }
    }
  }, [content, isLoading]);

  useEffect(() => {
    return () => abortControllerRef.current?.abort();
  }, []);

  const onFormSubmit = async (promptOverride?: string) => {
    const nextPrompt = (promptOverride ?? input).trim();

    if (!nextPrompt || isLoading) {
      setPromptError(true);
      window.setTimeout(() => setPromptError(false), 1000);
      return;
    }

    abortControllerRef.current?.abort();
    const abortController = new AbortController();
    abortControllerRef.current = abortController;

    setInput(nextPrompt);
    setLastPrompt(nextPrompt);
    setIsLoading(true);
    setIsValidJson(false);
    setParsedConfig(null);
    setContent("");
    setBannerState({ visible: false, type: "info", title: "" });

    try {
      const response = await fetch("/api/agent/build", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ prompt: nextPrompt }),
        signal: abortController.signal,
      });

      if (!response.ok) {
        throw new Error("The builder could not generate a configuration.");
      }

      if (!response.body) {
        throw new Error("Streaming is not available for this response.");
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let aggregated = "";

      while (true) {
        const { done, value } = await reader.read();

        if (done) {
          aggregated += decoder.decode();
          setContent(aggregated);
          break;
        }

        aggregated += decoder.decode(value, { stream: true });
        setContent(aggregated);
      }
    } catch (error) {
      if ((error as Error).name === "AbortError") {
        setBannerState({
          visible: true,
          type: "info",
          title: "Generation cancelled",
          message: "You can edit the prompt and try again.",
        });
      } else {
        setBannerState({
          visible: true,
          type: "error",
          title: "Failed to generate config",
          message:
            "Check your prompt and retry. The previous text has been preserved.",
        });
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleManualSave = (finalContent: string) => {
    const result = parseAgentConfig(finalContent);

    if (!result.success) {
      setBannerState({
        visible: true,
        type: "error",
        title: "Config is still invalid",
        message: "Fix the highlighted JSON before saving.",
      });
      return;
    }

    const savedAt = new Date().toISOString();
    localStorage.setItem(
      "agent-builder:last-config",
      JSON.stringify({
        prompt: lastPrompt,
        savedAt,
        config: result.data,
      }),
    );

    const file = new Blob([JSON.stringify(result.data, null, 2)], {
      type: "application/json",
    });
    const downloadUrl = URL.createObjectURL(file);
    const link = document.createElement("a");
    link.href = downloadUrl;
    link.download = `${result.data.id}.json`;
    link.click();
    URL.revokeObjectURL(downloadUrl);

    setBannerState({
      visible: true,
      type: "info",
      title: "Saved locally",
      message:
        "Downloaded a JSON copy and stored the latest draft in your browser.",
    });
  };

  const handleCancelGeneration = () => {
    abortControllerRef.current?.abort();
  };

  return (
    <div className="flex h-[calc(100vh-3rem)] w-full flex-col overflow-hidden bg-zinc-50">
      <main className="mx-auto flex h-full w-full max-w-[1600px] min-w-[1280px] flex-1 flex-col overflow-hidden lg:flex-row">
        {viewMode === "visual" ? (
          <div className="relative h-full w-full flex-1">
            <WorkflowCanvas onReturn={() => setViewMode("nl")} />
          </div>
        ) : (
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

                <div className="mb-6 z-20">
                  <StatusBanners
                    {...bannerState}
                    action={
                      bannerState.type === "error" ||
                      bannerState.type === "timeout"
                        ? {
                            label:
                              bannerState.type === "timeout"
                                ? "Cancel"
                                : "Retry",
                            onClick:
                              bannerState.type === "timeout"
                                ? handleCancelGeneration
                                : () => {
                                    void onFormSubmit(lastPrompt || input);
                                  },
                          }
                        : undefined
                    }
                  />
                </div>

                <form
                  onSubmit={(event) => {
                    event.preventDefault();
                    void onFormSubmit();
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
                  onClick={() => setViewMode("visual")}
                  className="flex items-center gap-2 rounded-lg border bg-white px-3 py-1.5 text-xs font-medium text-zinc-600 shadow-sm transition-colors hover:bg-zinc-50"
                >
                  <LayoutGrid className="h-3.5 w-3.5" />
                  Visual Canvas
                </button>
              </div>
              <div className="h-[85vh] max-h-[900px] w-full">
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
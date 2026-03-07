"use client";

import React, { useEffect, useRef, useState } from "react";
import Link from "next/link";
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

export default function AgentBuilderPage() {
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
    <div className="h-screen w-full bg-zinc-50 flex flex-col items-center overflow-hidden">
      {/* Top Navigation / Header */}
      <header className="w-full h-14 border-b bg-white flex items-center justify-between px-6 shrink-0 z-50">
        <div className="flex items-center gap-3">
          <div className="flex h-6 w-6 items-center justify-center rounded-md bg-zinc-900">
            <LayoutGrid className="h-3 w-3 text-white" />
          </div>
          <span className="text-sm font-semibold">Internal Agent Builder</span>
        </div>
        <div className="flex gap-3">
          <Link
            href="/"
            className="text-xs font-medium text-zinc-500 transition-colors hover:text-zinc-900"
          >
            Home
          </Link>
        </div>
      </header>

      {/* Main Workspace */}
      <main className="flex-1 w-full flex flex-col lg:flex-row overflow-hidden max-w-[1600px] mx-auto min-w-[1280px]">
        {viewMode === "visual" ? (
          <div className="w-full h-full flex-1 relative">
            <WorkflowCanvas onReturn={() => setViewMode("nl")} />
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
                  Describe your workflow in plain English. We'll automatically
                  wire the integrations and logic.
                </p>

                {/* Error / Status Displays */}
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

                {/* Prompt Form */}
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

            {/* Right Side: Preview Pane */}
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

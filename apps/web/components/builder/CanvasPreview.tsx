import { CheckCircle2, Clock3, GitBranch, PlayCircle, ShieldAlert, Webhook } from "lucide-react";

import type { BuilderAgentConfig } from "@/lib/agent-config";
import { extractDetectedTools } from "@/lib/agent-config";

interface CanvasPreviewProps {
  config: BuilderAgentConfig | null;
  isStreaming: boolean;
}

function getTriggerLabel(config: BuilderAgentConfig) {
  if ("Schedule" in config.trigger) {
    return `Schedule · ${config.trigger.Schedule.cron}`;
  }

  return `Event · ${config.trigger.Event.source}.${config.trigger.Event.event}`;
}

export function CanvasPreview({ config, isStreaming }: CanvasPreviewProps) {
  const detectedTools = extractDetectedTools(config);

  if (!config) {
    return (
      <div className="flex h-full min-h-[480px] items-center justify-center rounded-3xl border border-dashed border-zinc-300 bg-white/70 p-10 text-center shadow-sm">
        <div className="max-w-sm space-y-3">
          <PlayCircle className="mx-auto h-10 w-10 text-zinc-400" />
          <h3 className="text-lg font-semibold text-zinc-900">Visual workflow preview</h3>
          <p className="text-sm leading-6 text-zinc-500">
            Generate an agent first, then switch back here to review the trigger and execution steps as a simple workflow canvas.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-[480px] flex-col rounded-3xl border border-zinc-200 bg-white p-6 shadow-sm">
      <div className="flex flex-wrap items-start justify-between gap-4 border-b border-zinc-200 pb-5">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.28em] text-indigo-500">Workflow canvas</p>
          <h3 className="mt-2 text-2xl font-semibold text-zinc-950">{config.name}</h3>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-zinc-500">{config.summary}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          {detectedTools.length > 0 ? (
            detectedTools.map((tool) => (
              <span
                key={tool}
                className="inline-flex items-center gap-2 rounded-full border border-indigo-200 bg-indigo-50 px-3 py-1.5 text-xs font-medium text-indigo-700"
              >
                <Webhook className="h-3.5 w-3.5" />
                {tool}
              </span>
            ))
          ) : (
            <span className="inline-flex items-center rounded-full border border-amber-200 bg-amber-50 px-3 py-1.5 text-xs font-medium text-amber-700">
              No integrations detected yet
            </span>
          )}
        </div>
      </div>

      <div className="grid flex-1 gap-6 pt-6 xl:grid-cols-[260px_minmax(0,1fr)]">
        <aside className="rounded-2xl border border-zinc-200 bg-zinc-50 p-4">
          <div className="flex items-center gap-3">
            <div className="rounded-2xl bg-indigo-100 p-3 text-indigo-600">
              <Clock3 className="h-5 w-5" />
            </div>
            <div>
              <p className="text-sm font-semibold text-zinc-900">Trigger</p>
              <p className="text-xs text-zinc-500">{getTriggerLabel(config)}</p>
            </div>
          </div>
          <div className="mt-6 rounded-2xl border border-zinc-200 bg-white p-4 text-sm leading-6 text-zinc-600">
            {"Schedule" in config.trigger
              ? "This agent runs on a recurring schedule and then executes each step in order."
              : "This agent waits for an event and starts when the source system emits the configured trigger."}
          </div>
        </aside>

        <div className="overflow-auto rounded-2xl border border-zinc-200 bg-zinc-50/80 p-6">
          <div className="flex min-w-[640px] items-stretch gap-4">
            {config.steps.map((step, index) => {
              const isLast = index === config.steps.length - 1;

              return (
                <div key={step.id} className="flex items-center gap-4">
                  <article className="w-64 rounded-3xl border border-zinc-200 bg-white p-5 shadow-sm">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="text-xs font-semibold uppercase tracking-[0.24em] text-zinc-400">Step {index + 1}</p>
                        <h4 className="mt-2 text-base font-semibold text-zinc-950">{step.name}</h4>
                      </div>
                      <span className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2 py-1 text-[11px] font-semibold text-emerald-700">
                        {isStreaming ? <Clock3 className="h-3.5 w-3.5" /> : <CheckCircle2 className="h-3.5 w-3.5" />}
                        {isStreaming ? "Building" : "Ready"}
                      </span>
                    </div>

                    {step.tool ? (
                      <div className="mt-4 inline-flex items-center gap-2 rounded-full border border-indigo-200 bg-indigo-50 px-3 py-1 text-xs font-medium text-indigo-700">
                        <Webhook className="h-3.5 w-3.5" />
                        {step.tool}
                      </div>
                    ) : null}

                    {step.description ? (
                      <p className="mt-4 text-sm leading-6 text-zinc-600">{step.description}</p>
                    ) : null}

                    {step.approvalRequired ? (
                      <div className="mt-4 inline-flex items-center gap-2 rounded-full border border-amber-200 bg-amber-50 px-3 py-1 text-xs font-medium text-amber-700">
                        <ShieldAlert className="h-3.5 w-3.5" />
                        Human approval required
                      </div>
                    ) : null}
                  </article>

                  {!isLast ? (
                    <div className="flex min-w-10 items-center justify-center text-zinc-400">
                      <GitBranch className="h-5 w-5 rotate-90" />
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

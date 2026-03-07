import type { AgentConfig } from "@schema-types";
import { z } from "zod";

const toolNames = ["Slack", "GitHub", "Jira", "Notion", "Salesforce"] as const;

const stepSchema = z
  .object({
    id: z.string().min(1),
    name: z.string().min(1),
    tool: z.string().min(1).optional(),
    description: z.string().min(1).optional(),
    approvalRequired: z.boolean().optional(),
  })
  .passthrough();

const triggerSchema = z.union([
  z.object({
    Schedule: z.object({
      cron: z.string().min(1),
    }),
  }),
  z.object({
    Event: z.object({
      event: z.string().min(1),
      source: z.string().min(1),
    }),
  }),
]);

export const agentConfigSchema = z
  .object({
    id: z.string().min(1),
    name: z.string().min(1),
    trigger: triggerSchema,
    steps: z.array(stepSchema).min(1),
    summary: z.string().min(1).optional(),
  })
  .passthrough();

export type BuilderAgentStep = z.infer<typeof stepSchema>;
export type BuilderAgentConfig = Omit<AgentConfig, "steps"> &
  z.infer<typeof agentConfigSchema> & {
    steps: BuilderAgentStep[];
  };

export function parseAgentConfig(raw: string) {
  try {
    const parsed = JSON.parse(raw);
    return agentConfigSchema.safeParse(parsed);
  } catch {
    return {
      success: false as const,
      error: new z.ZodError([
        {
          code: "custom",
          message: "Configuration is not valid JSON.",
          path: [],
        },
      ]),
    };
  }
}

export function extractDetectedTools(config: Partial<BuilderAgentConfig> | null | undefined) {
  if (!config) {
    return [] as string[];
  }

  const haystack = JSON.stringify(config).toLowerCase();
  return toolNames.filter((tool) => haystack.includes(tool.toLowerCase()));
}

function titleCase(value: string) {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

function slugify(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
}

function guessTrigger(prompt: string, primaryTool: string | null) {
  const normalized = prompt.toLowerCase();

  if (normalized.includes("every friday")) {
    return { Schedule: { cron: "0 9 * * FRI" } };
  }

  if (normalized.includes("weekly") || normalized.includes("every week")) {
    return { Schedule: { cron: "0 9 * * MON" } };
  }

  if (normalized.includes("daily") || normalized.includes("every day")) {
    return { Schedule: { cron: "0 9 * * *" } };
  }

  const source = primaryTool?.toLowerCase() ?? "workspace";
  return {
    Event: {
      event: normalized.includes("alert") ? "item.changed" : "manual.requested",
      source,
    },
  };
}

function makeStep(id: number, name: string, tool?: string, description?: string, approvalRequired?: boolean) {
  return {
    id: `step-${id}`,
    name,
    ...(tool ? { tool } : {}),
    ...(description ? { description } : {}),
    ...(approvalRequired ? { approvalRequired } : {}),
  };
}

export function generateAgentConfigFromPrompt(prompt: string): BuilderAgentConfig {
  const normalized = prompt.trim().toLowerCase();
  const detectedTools = toolNames.filter((tool) => normalized.includes(tool.toLowerCase()));
  const primaryTool = detectedTools[0] ?? null;
  const promptTitle = titleCase(prompt.trim().replace(/[.?!]+$/, ""));
  const agentName = promptTitle || "New Agent Workflow";
  const slug = slugify(agentName) || "new-agent-workflow";

  const steps: BuilderAgentStep[] = [];

  if (detectedTools.length > 0) {
    steps.push(
      makeStep(
        1,
        `Collect context from ${detectedTools[0]}`,
        detectedTools[0],
        `Gather the source data required to execute: ${prompt.trim()}.`,
      ),
    );
  } else {
    steps.push(
      makeStep(
        1,
        "Collect workflow context",
        undefined,
        `Capture the inputs needed to execute: ${prompt.trim()}.`,
      ),
    );
  }

  steps.push(
    makeStep(
      2,
      normalized.includes("summarize") ? "Generate summary" : "Plan execution",
      normalized.includes("github") ? "GitHub" : undefined,
      "Transform the collected context into a structured plan for the agent.",
    ),
  );

  if (detectedTools.length > 1) {
    steps.push(
      makeStep(
        3,
        `Send result to ${detectedTools[1]}`,
        detectedTools[1],
        `Deliver the final output using ${detectedTools[1]}.`,
        normalized.includes("approve") || normalized.includes("review"),
      ),
    );
  } else if (primaryTool) {
    steps.push(
      makeStep(
        3,
        `Update ${primaryTool}`,
        primaryTool,
        `Publish the final result back into ${primaryTool}.`,
        normalized.includes("approve") || normalized.includes("review"),
      ),
    );
  } else {
    steps.push(
      makeStep(
        3,
        "Share the outcome",
        undefined,
        "Deliver the final output to the user or downstream system.",
      ),
    );
  }

  return {
    id: `agent-${slug}`,
    name: agentName,
    summary: prompt.trim(),
    trigger: guessTrigger(prompt, primaryTool),
    steps,
  } satisfies BuilderAgentConfig;
}

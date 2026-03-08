import type { AgentConfig } from "@schema-types";
import { z } from "zod";

const connectorDefinitions = [
  {
    id: "google_workspace",
    displayName: "Google Workspace",
    keywords: [
      "google",
      "gmail",
      "calendar",
      "drive",
      "docs",
      "sheets",
      "slides",
      "email",
      "meeting",
      "inbox",
    ],
  },
  {
    id: "notion",
    displayName: "Notion",
    keywords: ["notion", "database", "page", "doc", "document", "notes"],
  },
] as const;

const connectorNames = connectorDefinitions.map(
  (connector) => connector.displayName,
) as (typeof connectorDefinitions)[number]["displayName"][];

const stepSchema = z
  .object({
    id: z.string().min(1),
    name: z.string().min(1),
    tool_name: z.string().min(1).optional(),
    tool: z.string().min(1).optional(),
    description: z.string().min(1).optional(),
    requires_approval: z.boolean().optional(),
    approvalRequired: z.boolean().optional(),
  })
  .passthrough()
  .transform((step) => ({
    ...step,
    tool_name: step.tool_name ?? step.tool,
    requires_approval: step.requires_approval ?? step.approvalRequired ?? false,
  }));

const triggerSchema = z.union([
  z.literal("Manual"),
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

export function extractDetectedTools(
  config: Partial<BuilderAgentConfig> | null | undefined,
) {
  if (!config) {
    return [] as string[];
  }

  const haystack = JSON.stringify(config).toLowerCase();
  const detected = new Set<string>();

  for (const step of config.steps ?? []) {
    if (step.tool_name?.startsWith("google_")) {
      detected.add("Google Workspace");
    }

    if (step.tool_name?.startsWith("notion_")) {
      detected.add("Notion");
    }
  }

  for (const connector of connectorDefinitions) {
    if (connector.keywords.some((keyword) => haystack.includes(keyword))) {
      detected.add(connector.displayName);
    }
  }

  return connectorNames.filter((tool) => detected.has(tool));
}

type ConnectorDefinition = (typeof connectorDefinitions)[number];

function detectConnectors(value: string): ConnectorDefinition[] {
  const normalized = value.toLowerCase();
  return connectorDefinitions.filter((connector) =>
    connector.keywords.some((keyword) => normalized.includes(keyword)),
  );
}

function containsEmailAddress(value: string) {
  return value
    .split(/\s+/)
    .map((part) => part.replace(/^[<>()\[\]{}"'.,;:]+|[<>()\[\]{}"'.,;:]+$/g, ""))
    .some((part) => {
      const atIndex = part.indexOf("@");
      return atIndex > 0 && atIndex < part.length - 1 && part.slice(atIndex + 1).includes(".");
    });
}

function containsMessageContent(value: string) {
  const normalized = value.toLowerCase();
  return (
    [" saying ", " that says ", " with body ", " body ", " message ", " subject "].some(
      (marker) => normalized.includes(marker),
    ) ||
    (value.match(/"/g)?.length ?? 0) >= 2 ||
    (value.match(/'/g)?.length ?? 0) >= 2
  );
}

function canSendGmailFromPrompt(value: string) {
  return containsEmailAddress(value) && containsMessageContent(value);
}

function inferGoogleToolName(
  value: string,
  mode: "read" | "write" = "read",
) {
  const normalized = value.toLowerCase();

  if (
    normalized.includes("gmail") ||
    normalized.includes("email") ||
    normalized.includes("inbox")
  ) {
    if (
      (mode === "write" ||
      normalized.includes("send") ||
      normalized.includes("reply") ||
      normalized.includes("draft"))
    ) {
      return canSendGmailFromPrompt(value) ? "google_send_gmail" : undefined;
    }

    if (
      normalized.includes("search") ||
      normalized.includes("find") ||
      normalized.includes("query")
    ) {
      return "google_search_gmail";
    }

    const explicitlyReadsGmail =
      normalized.includes("search") ||
      normalized.includes("find") ||
      normalized.includes("query") ||
      normalized.includes("list") ||
      normalized.includes("retrieve") ||
      normalized.includes("open") ||
      normalized.includes("review") ||
      normalized.includes("inspect") ||
      normalized.includes("read") ||
      normalized.includes("collect") ||
      normalized.includes("inbox");

    return explicitlyReadsGmail ? "google_list_gmail_messages" : undefined;
  }

  if (
    normalized.includes("calendar") ||
    normalized.includes("meeting") ||
    normalized.includes("event")
  ) {
    if (
      mode === "write" ||
      normalized.includes("schedule") ||
      normalized.includes("book") ||
      normalized.includes("create") ||
      normalized.includes("add")
    ) {
      return "google_create_calendar_event";
    }

    if (
      normalized.includes("event") &&
      (normalized.includes("get") || normalized.includes("retrieve"))
    ) {
      return "google_get_calendar_event";
    }

    if (normalized.includes("list calendars") || normalized.includes("all calendars")) {
      return "google_list_calendars";
    }

    return "google_list_calendar_events";
  }

  if (
    normalized.includes("drive") ||
    normalized.includes("file") ||
    normalized.includes("docs") ||
    normalized.includes("sheets") ||
    normalized.includes("slides")
  ) {
    if (normalized.includes("export") || normalized.includes("download")) {
      return "google_export_drive_file";
    }

    if (
      normalized.includes("get") ||
      normalized.includes("open") ||
      normalized.includes("read")
    ) {
      return "google_get_drive_file";
    }

    if (
      normalized.includes("search") ||
      normalized.includes("find") ||
      normalized.includes("query")
    ) {
      return "google_search_drive";
    }

    return "google_list_drive_files";
  }

  return undefined;
}

function inferNotionToolName(
  value: string,
  mode: "read" | "write" = "read",
) {
  const normalized = value.toLowerCase();

  if (normalized.includes("update") || normalized.includes("edit")) {
    return "notion_update_page";
  }

  if (
    normalized.includes("append") ||
    normalized.includes("block") ||
    normalized.includes("content")
  ) {
    return "notion_append_block_children";
  }

  if (
    normalized.includes("database") ||
    normalized.includes("query") ||
    normalized.includes("filter")
  ) {
    return "notion_query_database";
  }

  if (normalized.includes("search") || normalized.includes("find")) {
    return "notion_search";
  }

  if (
    mode === "write" ||
    normalized.includes("create") ||
    normalized.includes("page") ||
    normalized.includes("document") ||
    normalized.includes("note")
  ) {
    return "notion_create_page";
  }

  return "notion_search";
}

function inferToolName(
  value: string,
  preferredConnectorId?: ConnectorDefinition["id"],
  mode: "read" | "write" = "read",
) {
  const preferredConnector = preferredConnectorId
    ? connectorDefinitions.find((connector) => connector.id === preferredConnectorId)
    : detectConnectors(value)[0];

  if (!preferredConnector) {
    return undefined;
  }

  if (preferredConnector.id === "google_workspace") {
    return inferGoogleToolName(value, mode);
  }

  return inferNotionToolName(value, mode);
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

function guessTrigger(
  prompt: string,
  primaryConnectorId: ConnectorDefinition["id"] | null,
) {
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

  if (normalized.includes("manual") || normalized.includes("on demand")) {
    return "Manual" as const;
  }

  if (primaryConnectorId === "google_workspace") {
    return {
      Event: {
        event:
          normalized.includes("gmail") || normalized.includes("email")
            ? "gmail.message.received"
            : normalized.includes("calendar") || normalized.includes("meeting")
              ? "calendar.event.updated"
              : "drive.file.updated",
        source: "google_workspace",
      },
    };
  }

  if (primaryConnectorId === "notion") {
    return {
      Event: {
        event: normalized.includes("database") ? "database.item.updated" : "page.updated",
        source: "notion",
      },
    };
  }

  return {
    Event: {
      event: normalized.includes("alert") ? "item.changed" : "manual.requested",
      source: "workspace",
    },
  };
}

function makeStep(
  id: number,
  name: string,
  toolName?: string,
  description?: string,
  requiresApproval?: boolean,
) {
  return {
    id: `step-${id}`,
    name,
    tool_name: toolName,
    ...(description ? { description } : {}),
    requires_approval: Boolean(requiresApproval),
  };
}

export function generateAgentConfigFromPrompt(
  prompt: string,
): BuilderAgentConfig {
  const normalized = prompt.trim().toLowerCase();
  const detectedConnectors = detectConnectors(prompt);
  const primaryConnector = detectedConnectors[0] ?? null;
  const destinationConnector = detectedConnectors[1] ?? primaryConnector;
  const promptTitle = titleCase(prompt.trim().replace(/[.?!]+$/, ""));
  const agentName = promptTitle || "New Agent Workflow";
  const slug = slugify(agentName) || "new-agent-workflow";

  const steps: BuilderAgentStep[] = [];
  const readTool = inferToolName(prompt, primaryConnector?.id, "read");
  const writeTool = inferToolName(prompt, destinationConnector?.id, "write");

  if (primaryConnector && readTool) {
    steps.push(
      makeStep(
        1,
        `Collect context from ${primaryConnector.displayName}`,
        readTool,
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
      normalized.includes("summarize")
        ? "Generate summary content"
        : "Generate draft content",
      "generate_content",
      "Transform the collected context into a structured plan for the agent.",
    ),
  );

  if (destinationConnector && writeTool) {
    steps.push(
      makeStep(
        3,
        `Update ${destinationConnector.displayName}`,
        writeTool,
        `Deliver the final output using ${destinationConnector.displayName}.`,
        normalized.includes("approve") || normalized.includes("review"),
      ),
    );
  } else if (
    normalized.includes("gmail") ||
    normalized.includes("email") ||
    normalized.includes("send") ||
    normalized.includes("reply")
  ) {
    steps.push(
      makeStep(
        3,
        "Request recipient and delivery details",
        undefined,
        "Collect the missing recipient or message details before sending the email.",
        false,
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
    trigger: guessTrigger(prompt, primaryConnector?.id ?? null),
    steps,
  } satisfies BuilderAgentConfig;
}

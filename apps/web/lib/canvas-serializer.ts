import type { Edge, Node } from "@xyflow/react";
import type { AgentConfig, AgentStep, Trigger } from "@schema-types";

export interface WorkflowNodeData extends Record<string, unknown> {
  label?: string;
  tool?: string;
  description?: string;
  timeout?: number | string;
  requires_approval?: boolean;
  triggerType?: "manual" | "schedule" | "event";
  cron?: string;
  eventSource?: string;
  eventName?: string;
}

export interface SerializableRunStep {
  name: string;
  requires_approval: boolean;
  tool_name?: string;
  tool_arguments?: Record<string, unknown>;
}

function makeUuid() {
  if (
    typeof crypto !== "undefined" &&
    typeof crypto.randomUUID === "function"
  ) {
    return crypto.randomUUID();
  }

  return `workflow-${Math.random().toString(36).slice(2)}-${Date.now()}`;
}

function getNodeData(node: Node): WorkflowNodeData {
  return (node.data ?? {}) as WorkflowNodeData;
}

function buildMaps(nodes: Node[], edges: Edge[]) {
  const nodeMap = new Map(nodes.map((node) => [node.id, node]));
  const outgoing = new Map<string, Edge[]>();
  const incoming = new Map<string, Edge[]>();

  nodes.forEach((node) => {
    outgoing.set(node.id, []);
    incoming.set(node.id, []);
  });

  edges.forEach((edge) => {
    outgoing.get(edge.source)?.push(edge);
    incoming.get(edge.target)?.push(edge);
  });

  return { nodeMap, outgoing, incoming };
}

function getTriggerNode(nodes: Node[]) {
  const triggerNodes = nodes.filter((node) => node.type === "trigger");

  if (triggerNodes.length !== 1) {
    throw new Error("Exactly one trigger node is required.");
  }

  return triggerNodes[0];
}

function orderWorkflowNodes(nodes: Node[], edges: Edge[]) {
  const triggerNode = getTriggerNode(nodes);
  const { nodeMap, outgoing, incoming } = buildMaps(nodes, edges);
  const ordered: Node[] = [];
  const visited = new Set<string>([triggerNode.id]);
  let currentNode = triggerNode;

  while (true) {
    const nextEdges = (outgoing.get(currentNode.id) ?? []).filter((edge) =>
      nodeMap.has(edge.target),
    );

    if (nextEdges.length > 1) {
      throw new Error(
        `Branching workflows are not supported by the backend yet. "${getNodeData(currentNode).label ?? currentNode.id}" has multiple outgoing paths.`,
      );
    }

    if (nextEdges.length === 0) {
      break;
    }

    const nextNode = nodeMap.get(nextEdges[0].target);
    if (!nextNode) {
      break;
    }

    const nextIncoming = incoming.get(nextNode.id) ?? [];
    if (nextIncoming.length > 1) {
      throw new Error(
        `Merged paths are not supported by the backend yet. "${getNodeData(nextNode).label ?? nextNode.id}" has multiple incoming paths.`,
      );
    }

    if (visited.has(nextNode.id)) {
      throw new Error("Cycle detected while serializing the workflow.");
    }

    visited.add(nextNode.id);
    ordered.push(nextNode);
    currentNode = nextNode;
  }

  const unvisited = nodes.filter(
    (node) => node.id !== triggerNode.id && !visited.has(node.id),
  );

  if (unvisited.length > 0) {
    throw new Error(
      `Some nodes are disconnected from the trigger. Connect or remove "${getNodeData(unvisited[0]).label ?? unvisited[0].id}".`,
    );
  }

  return ordered;
}

function serializeTrigger(node: Node): Trigger {
  const data = getNodeData(node);

  if (data.triggerType === "schedule" || data.cron) {
    return {
      Schedule: {
        cron: data.cron?.trim() || "0 9 * * *",
      },
    };
  }

  if (data.triggerType === "event" || data.eventSource || data.eventName) {
    return {
      Event: {
        source: data.eventSource?.trim() || "workspace",
        event: data.eventName?.trim() || "manual.requested",
      },
    };
  }

  return "Manual";
}

function inferToolName(node: Node, data: WorkflowNodeData) {
  if (node.type === "human") {
    return "human.request_approval";
  }

  if (node.type === "condition") {
    return "workflow.condition";
  }

  if (node.type === "step") {
    return data.tool?.trim() || undefined;
  }

  return undefined;
}

function nodeToAgentStep(node: Node): AgentStep | null {
  if (node.type === "trigger" || node.type === "output") {
    return null;
  }

  const data = getNodeData(node);
  return {
    id: makeUuid(),
    name: data.label?.trim() || `${node.type ?? "step"} step`,
    tool_name: inferToolName(node, data),
    requires_approval: node.type === "human" || Boolean(data.requires_approval),
  };
}

export function serializeCanvasToAgentConfig(
  nodes: Node[],
  edges: Edge[],
  name: string,
): AgentConfig {
  const triggerNode = getTriggerNode(nodes);
  const orderedNodes = orderWorkflowNodes(nodes, edges);
  const steps = orderedNodes
    .map(nodeToAgentStep)
    .filter((step): step is AgentStep => step !== null);

  if (steps.length === 0) {
    throw new Error("Add at least one workflow step before saving.");
  }

  return {
    id: makeUuid(),
    name: name.trim() || "Untitled Agent",
    trigger: serializeTrigger(triggerNode),
    steps,
  };
}

export function serializeCanvasToRunSteps(
  nodes: Node[],
  edges: Edge[],
): SerializableRunStep[] {
  return orderWorkflowNodes(nodes, edges)
    .filter((node) => node.type !== "trigger" && node.type !== "output")
    .map((node) => {
      const data = getNodeData(node);
      const toolName = inferToolName(node, data);

      return {
        name: data.label?.trim() || `${node.type ?? "step"} step`,
        requires_approval:
          node.type === "human" || Boolean(data.requires_approval),
        ...(toolName ? { tool_name: toolName } : {}),
        ...(toolName
          ? {
              tool_arguments: {
                label: data.label ?? null,
                description: data.description ?? null,
                timeout_ms:
                  data.timeout !== undefined && data.timeout !== ""
                    ? Number(data.timeout)
                    : null,
                node_type: node.type ?? "step",
              },
            }
          : {}),
      };
    });
}

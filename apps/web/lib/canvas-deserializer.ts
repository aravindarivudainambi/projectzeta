import type { Edge, Node } from "@xyflow/react";
import type { AgentConfig } from "@schema-types";

import type { WorkflowNodeData } from "@/lib/canvas-serializer";

function triggerDataFromConfig(config: AgentConfig): WorkflowNodeData {
  if (config.trigger === "Manual") {
    return {
      label: "Manual Trigger",
      triggerType: "manual",
    };
  }

  if ("Schedule" in config.trigger) {
    return {
      label: "Scheduled Trigger",
      triggerType: "schedule",
      cron: config.trigger.Schedule.cron,
    };
  }

  return {
    label: "Event Trigger",
    triggerType: "event",
    eventSource: config.trigger.Event.source,
    eventName: config.trigger.Event.event,
  };
}

function nodeTypeFromStep(step: AgentConfig["steps"][number]): Node["type"] {
  if (step.tool_name === "human.request_approval" || step.requires_approval) {
    return "human";
  }

  if (step.tool_name === "workflow.condition") {
    return "condition";
  }

  return "step";
}

export function deserializeAgentConfigToCanvas(config: AgentConfig): {
  nodes: Node[];
  edges: Edge[];
} {
  const triggerNode: Node = {
    id: "trigger-node",
    type: "trigger",
    position: { x: 250, y: 50 },
    data: triggerDataFromConfig(config),
  };

  const stepNodes: Node[] = config.steps.map((step, index) => ({
    id: step.id,
    type: nodeTypeFromStep(step),
    position: { x: 250, y: 220 + index * 160 },
    data: {
      label: step.name,
      tool: step.tool_name,
      requires_approval: step.requires_approval,
      description: "",
      timeout: 30000,
    },
  }));

  const outputNode: Node = {
    id: "output-node",
    type: "output",
    position: { x: 250, y: 220 + config.steps.length * 160 },
    data: { label: "Workflow Complete" },
  };

  const allNodes = [triggerNode, ...stepNodes, outputNode];
  const edges: Edge[] = [];

  for (let index = 0; index < allNodes.length - 1; index += 1) {
    edges.push({
      id: `edge-${allNodes[index].id}-${allNodes[index + 1].id}`,
      source: allNodes[index].id,
      target: allNodes[index + 1].id,
    });
  }

  return { nodes: allNodes, edges };
}

"use client";

import React, { useState, useCallback, useRef, useEffect } from "react";
import { useRouter } from "next/navigation";
import {
  ReactFlow,
  ReactFlowProvider,
  addEdge,
  useNodesState,
  useEdgesState,
  Controls,
  MiniMap,
  Background,
  MarkerType,
  Connection,
  Edge,
  Node,
  useReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { createAgent, startRun } from "@api-client";
import type { AgentConfig } from "@schema-types";

import { nodeTypes } from "./CustomNode";
import { NodePalette } from "./NodePalette";
import { NodeSidebar } from "./NodeSidebar";
import { Save, AlertCircle, CheckCircle2, PlayCircle } from "lucide-react";
import {
  serializeCanvasToAgentConfig,
  serializeCanvasToRunSteps,
} from "@/lib/canvas-serializer";
import { deserializeAgentConfigToCanvas } from "@/lib/canvas-deserializer";

const initialNodes: Node[] = [
  {
    id: "trigger-1",
    type: "trigger",
    position: { x: 250, y: 50 },
    data: { label: "Workflow Start", triggerType: "manual" },
  },
];

let id = 0;
const getId = () => `dndnode_${id++}`;

function CanvasEditor({
  onReturn,
  initialConfig,
}: {
  onReturn: () => void;
  initialConfig?: AgentConfig | null;
}) {
  const router = useRouter();
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const { screenToFlowPosition } = useReactFlow();

  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [agentName, setAgentName] = useState(
    initialConfig?.name ?? "Untitled Agent",
  );
  const [savedAgentId, setSavedAgentId] = useState<string | null>(
    initialConfig?.id ?? null,
  );
  const [isSaving, setIsSaving] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [validationState, setValidationState] = useState<{
    status: "idle" | "error" | "success";
    message: string;
  }>({ status: "idle", message: "" });

  // History for undo/redo
  const [history, setHistory] = useState<{ nodes: Node[]; edges: Edge[] }[]>([
    { nodes: initialNodes, edges: [] },
  ]);
  const [historyIndex, setHistoryIndex] = useState(0);

  useEffect(() => {
    if (!initialConfig) {
      return;
    }

    const canvasState = deserializeAgentConfigToCanvas(initialConfig);
    setAgentName(initialConfig.name);
    setSavedAgentId(initialConfig.id);
    setNodes(canvasState.nodes);
    setEdges(canvasState.edges);
    setSelectedNode(null);
    setHistory([{ nodes: canvasState.nodes, edges: canvasState.edges }]);
    setHistoryIndex(0);
    setValidationState({ status: "idle", message: "Loaded from builder" });
  }, [initialConfig, setEdges, setNodes]);

  const takeSnapshot = useCallback(() => {
    setHistory((prev) => {
      const newHistory = prev.slice(0, historyIndex + 1);
      newHistory.push({ nodes, edges });
      if (newHistory.length > 50) newHistory.shift();
      return newHistory;
    });
    setHistoryIndex((prev) => Math.min(prev + 1, 50));
    setValidationState({ status: "idle", message: "Unsaved changes" });
  }, [nodes, edges, historyIndex]);

  const onConnect = useCallback(
    (params: Connection | Edge) => {
      setEdges((eds) =>
        addEdge(
          { ...params, markerEnd: { type: MarkerType.ArrowClosed } },
          eds,
        ),
      );
      takeSnapshot();
    },
    [setEdges, takeSnapshot],
  );

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const type = event.dataTransfer.getData("application/reactflow");
      const label = event.dataTransfer.getData("application/reactflow-label");

      if (!type) return;

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const newNode: Node = {
        id: getId(),
        type,
        position,
        data: {
          label,
          ...(type === "trigger" ? { triggerType: "manual" } : {}),
          ...(type === "step"
            ? { timeout: "30000", requires_approval: false }
            : {}),
          ...(type === "human" ? { requires_approval: true } : {}),
        },
      };

      setNodes((nds) => nds.concat(newNode));
      takeSnapshot();
    },
    [screenToFlowPosition, setNodes, takeSnapshot],
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  const onPaneClick = useCallback(() => {
    setSelectedNode(null);
  }, []);

  const onUpdateNode = useCallback(
    (id: string, data: any) => {
      setNodes((nds) =>
        nds.map((n) => {
          if (n.id === id) {
            const updatedNode = { ...n, data };
            if (selectedNode?.id === id) setSelectedNode(updatedNode);
            return updatedNode;
          }
          return n;
        }),
      );
      takeSnapshot();
    },
    [selectedNode, setNodes, takeSnapshot],
  );

  const onNodesDelete = useCallback(() => takeSnapshot(), [takeSnapshot]);
  const onEdgesDelete = useCallback(() => takeSnapshot(), [takeSnapshot]);

  // Undo / Redo keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key === "z") {
        if (event.shiftKey) {
          // Redo
          if (historyIndex < history.length - 1) {
            event.preventDefault();
            const nextIdx = historyIndex + 1;
            setNodes(history[nextIdx].nodes);
            setEdges(history[nextIdx].edges);
            setHistoryIndex(nextIdx);
          }
        } else {
          // Undo
          if (historyIndex > 0) {
            event.preventDefault();
            const prevIdx = historyIndex - 1;
            setNodes(history[prevIdx].nodes);
            setEdges(history[prevIdx].edges);
            setHistoryIndex(prevIdx);
          }
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [history, historyIndex, setNodes, setEdges]);

  const validateAndSave = async () => {
    // 1. Check for step nodes missing required tool field
    const invalidStepNodes = nodes.filter(
      (n) => n.type === "step" && !n.data.tool,
    );
    if (invalidStepNodes.length > 0) {
      setValidationState({
        status: "error",
        message: `Node "${invalidStepNodes[0].data.label}" is missing a required Tool Name.`,
      });
      setSelectedNode(invalidStepNodes[0]);
      return;
    }

    // 2. Check for orphaned nodes (only when more than one node exists)
    if (nodes.length > 1) {
      const connectedNodeIds = new Set<string>();
      edges.forEach((e) => {
        connectedNodeIds.add(e.source);
        connectedNodeIds.add(e.target);
      });

      const orphanedNodes = nodes.filter((n) => !connectedNodeIds.has(n.id));
      if (orphanedNodes.length > 0) {
        setValidationState({
          status: "error",
          message: `Orphaned node detected: "${orphanedNodes[0].data.label}". Connect or remove it.`,
        });
        setSelectedNode(orphanedNodes[0]);
        return;
      }
    }

    // 3. Cycle detection (DFS)
    const graph = new Map<string, string[]>();
    nodes.forEach((n) => graph.set(n.id, []));
    edges.forEach((e) => {
      const neighbors = graph.get(e.source);
      if (neighbors) neighbors.push(e.target);
    });

    let hasCycle = false;
    const visited = new Set<string>();
    const recursionStack = new Set<string>();

    const dfs = (nodeId: string): boolean => {
      visited.add(nodeId);
      recursionStack.add(nodeId);

      const neighbors = graph.get(nodeId) || [];
      for (const neighbor of neighbors) {
        if (!visited.has(neighbor)) {
          if (dfs(neighbor)) return true;
        } else if (recursionStack.has(neighbor)) {
          return true;
        }
      }

      recursionStack.delete(nodeId);
      return false;
    };

    for (const node of nodes) {
      if (!visited.has(node.id)) {
        if (dfs(node.id)) {
          hasCycle = true;
          break;
        }
      }
    }

    if (hasCycle) {
      setValidationState({
        status: "error",
        message: "Infinite loop (cycle) detected in the graph.",
      });
      return;
    }

    try {
      setIsSaving(true);
      const config = serializeCanvasToAgentConfig(nodes, edges, agentName);
      const savedAgent = await createAgent({
        name: config.name,
        trigger: config.trigger,
        steps: config.steps.map((step) => ({
          name: step.name,
          requires_approval: Boolean(step.requires_approval),
          ...(step.tool_name ? { tool_name: step.tool_name } : {}),
        })),
      });

      setSavedAgentId(savedAgent.id);
      setAgentName(savedAgent.name);
      setValidationState({
        status: "success",
        message: `Saved agent ${savedAgent.id.slice(0, 8)}…`,
      });
    } catch (error) {
      setValidationState({
        status: "error",
        message:
          error instanceof Error ? error.message : "Failed to save workflow.",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleRunAgent = async () => {
    if (!savedAgentId) {
      setValidationState({
        status: "error",
        message: "Save the agent before starting a run.",
      });
      return;
    }

    try {
      setIsRunning(true);
      const run = await startRun({
        agent_id: savedAgentId,
        steps: serializeCanvasToRunSteps(nodes, edges),
      });

      router.push(`/runs/${run.run_id}`);
    } catch (error) {
      setValidationState({
        status: "error",
        message:
          error instanceof Error ? error.message : "Failed to start run.",
      });
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <div className="flex h-full w-full bg-zinc-50 relative overflow-hidden">
      <NodePalette />

      <div
        className="flex flex-col flex-1 h-full relative"
        ref={reactFlowWrapper}
      >
        {/* Toolbar */}
        <div className="absolute top-4 left-1/2 -translate-x-1/2 z-10 flex items-center gap-4 bg-white px-4 py-2 rounded-full border shadow-sm">
          <div className="flex items-center gap-2 border-r pr-4">
            <button
              onClick={onReturn}
              className="text-sm font-medium text-zinc-600 hover:text-zinc-900 transition-colors"
            >
              Exit Canvas
            </button>
          </div>

          <input
            value={agentName}
            onChange={(event) => setAgentName(event.target.value)}
            className="min-w-[220px] rounded-md border border-zinc-200 bg-zinc-50 px-3 py-1.5 text-sm font-medium text-zinc-800 outline-none transition focus:border-zinc-400 focus:bg-white"
            placeholder="Agent name"
          />

          <div className="flex items-center gap-2 border-r pr-4 min-w-[160px]">
            {validationState.status === "error" && (
              <span className="flex items-center gap-1.5 text-xs font-medium text-red-600">
                <AlertCircle size={14} />
                {validationState.message}
              </span>
            )}
            {validationState.status === "success" && (
              <span className="flex items-center gap-1.5 text-xs font-medium text-emerald-600">
                <CheckCircle2 size={14} />
                Synced
              </span>
            )}
            {validationState.status === "idle" && (
              <span className="flex items-center gap-1.5 text-xs font-medium text-zinc-500">
                <div className="w-2 h-2 rounded-full bg-amber-400" />
                {validationState.message || "Unsaved"}
              </span>
            )}
          </div>

          <button
            onClick={validateAndSave}
            disabled={isSaving}
            className="flex items-center gap-2 bg-zinc-900 text-white px-3 py-1.5 text-sm font-medium rounded-md hover:bg-zinc-800 transition-colors"
          >
            <Save size={14} />
            {isSaving ? "Saving..." : "Validate & Save"}
          </button>

          <button
            onClick={handleRunAgent}
            disabled={!savedAgentId || isRunning}
            className="flex items-center gap-2 rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 transition-colors enabled:hover:bg-zinc-50 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <PlayCircle size={14} />
            {isRunning ? "Starting..." : "Run Agent"}
          </button>
        </div>

        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onDrop={onDrop}
          onDragOver={onDragOver}
          onNodeClick={onNodeClick}
          onPaneClick={onPaneClick}
          onNodesDelete={onNodesDelete}
          onEdgesDelete={onEdgesDelete}
          nodeTypes={nodeTypes}
          fitView
          minZoom={0.25}
          maxZoom={2}
          className="bg-zinc-50/50"
        >
          <Background color="#ccc" gap={16} />
          <Controls position="bottom-left" />
          <MiniMap
            nodeColor={(n) => {
              if (n.type === "trigger") return "#10b981";
              if (n.type === "step") return "#3b82f6";
              if (n.type === "condition") return "#f59e0b";
              if (n.type === "human") return "#a855f7";
              return "#71717a";
            }}
            maskColor="rgba(240, 240, 245, 0.6)"
            position="bottom-right"
            className="bg-white/80 backdrop-blur-sm border shadow-sm rounded-lg overflow-hidden m-4"
          />
        </ReactFlow>
      </div>

      <NodeSidebar selectedNode={selectedNode} onUpdateNode={onUpdateNode} />
    </div>
  );
}

export function WorkflowCanvas(props: {
  onReturn: () => void;
  initialConfig?: AgentConfig | null;
}) {
  return (
    <ReactFlowProvider>
      <CanvasEditor {...props} />
    </ReactFlowProvider>
  );
}

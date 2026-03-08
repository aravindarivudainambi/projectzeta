"use client";

import React from "react";
import type { Node } from "@xyflow/react";

interface NodeSidebarProps {
  selectedNode: Node | null;
  onUpdateNode: (id: string, data: any) => void;
}

export function NodeSidebar({ selectedNode, onUpdateNode }: NodeSidebarProps) {
  if (!selectedNode) {
    return (
      <div className="w-72 bg-white border-l h-full p-6 flex flex-col items-center justify-center text-center shadow-[-1px_0_10px_rgba(0,0,0,0.02)] z-10 shrink-0 text-zinc-400">
        <p className="text-sm">Select a node to edit its properties</p>
      </div>
    );
  }

  const handleChange = (field: string, value: string | boolean) => {
    onUpdateNode(selectedNode.id, { ...selectedNode.data, [field]: value });
  };

  return (
    <div className="w-72 bg-white flex flex-col border-l h-full shadow-[-1px_0_10px_rgba(0,0,0,0.02)] z-10 shrink-0">
      <div className="p-5 border-b sticky top-0 bg-white z-10">
        <h3 className="text-base font-semibold text-zinc-900 capitalize truncate">
          {selectedNode.type} Node
        </h3>
        <p className="text-xs text-zinc-500 mt-0.5 truncate">
          {selectedNode.id}
        </p>
      </div>

      <div className="p-5 overflow-y-auto flex flex-col gap-5 flex-1 pb-10">
        <div className="flex flex-col gap-2 relative">
          <label className="text-xs font-semibold text-zinc-700">Label</label>
          <input
            type="text"
            className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
            value={(selectedNode.data.label as string) || ""}
            onChange={(e) => handleChange("label", e.target.value)}
            placeholder="E.g. Get items"
          />
        </div>

        {selectedNode.type === "trigger" && (
          <>
            <div className="flex flex-col gap-2">
              <label className="text-xs font-semibold text-zinc-700">
                Trigger Type
              </label>
              <select
                className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                value={(selectedNode.data.triggerType as string) || "manual"}
                onChange={(e) => handleChange("triggerType", e.target.value)}
              >
                <option value="manual">Manual</option>
                <option value="schedule">Schedule</option>
                <option value="event">Event</option>
              </select>
            </div>

            {((selectedNode.data.triggerType as string) || "manual") ===
            "schedule" ? (
              <div className="flex flex-col gap-2">
                <label className="text-xs font-semibold text-zinc-700">
                  Cron
                </label>
                <input
                  type="text"
                  className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                  value={(selectedNode.data.cron as string) || "0 9 * * *"}
                  onChange={(e) => handleChange("cron", e.target.value)}
                  placeholder="0 9 * * *"
                />
              </div>
            ) : null}

            {((selectedNode.data.triggerType as string) || "manual") ===
            "event" ? (
              <>
                <div className="flex flex-col gap-2">
                  <label className="text-xs font-semibold text-zinc-700">
                    Event Source
                  </label>
                  <input
                    type="text"
                    className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                    value={
                      (selectedNode.data.eventSource as string) || "workspace"
                    }
                    onChange={(e) =>
                      handleChange("eventSource", e.target.value)
                    }
                    placeholder="google_workspace"
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <label className="text-xs font-semibold text-zinc-700">
                    Event Name
                  </label>
                  <input
                    type="text"
                    className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                    value={
                      (selectedNode.data.eventName as string) ||
                      "manual.requested"
                    }
                    onChange={(e) => handleChange("eventName", e.target.value)}
                    placeholder="page.updated"
                  />
                </div>
              </>
            ) : null}
          </>
        )}

        {selectedNode.type === "step" && (
          <>
            <div className="flex flex-col gap-2 relative">
              <label className="text-xs font-semibold text-zinc-700">
                Tool Name <span className="text-red-500">*</span>
              </label>
              <input
                type="text"
                className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                value={(selectedNode.data.tool as string) || ""}
                onChange={(e) => handleChange("tool", e.target.value)}
                placeholder="E.g. generate_content, google_send_gmail, or notion_create_page"
              />
              {!selectedNode.data.tool && (
                <p className="text-[10px] text-red-500 mt-1 absolute -bottom-4">
                  Required field missing
                </p>
              )}
            </div>
            <div className="flex flex-col gap-2">
              <label className="text-xs font-semibold text-zinc-700">
                Timeout (ms)
              </label>
              <input
                type="number"
                className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                value={(selectedNode.data.timeout as string) || "30000"}
                onChange={(e) => handleChange("timeout", e.target.value)}
              />
            </div>
          </>
        )}

        {selectedNode.type !== "trigger" && selectedNode.type !== "output" ? (
          <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm text-zinc-700">
            <span className="font-medium">Pause for approval</span>
            <input
              type="checkbox"
              className="h-4 w-4 rounded border-zinc-300 text-zinc-900 focus:ring-zinc-400"
              checked={Boolean(selectedNode.data.requires_approval)}
              onChange={(e) =>
                handleChange("requires_approval", e.target.checked)
              }
            />
          </label>
        ) : null}

        <div className="flex flex-col gap-2">
          <label className="text-xs font-semibold text-zinc-700">
            Description
          </label>
          <textarea
            className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white min-h-[80px] resize-y"
            value={(selectedNode.data.description as string) || ""}
            onChange={(e) => handleChange("description", e.target.value)}
            placeholder="Add some notes about this node..."
          />
        </div>
      </div>
    </div>
  );
}

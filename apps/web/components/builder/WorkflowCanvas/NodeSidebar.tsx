import React from 'react';
import type { Node } from '@xyflow/react';

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

  const handleChange = (field: string, value: string) => {
    onUpdateNode(selectedNode.id, { ...selectedNode.data, [field]: value });
  };

  return (
    <div className="w-72 bg-white flex flex-col border-l h-full shadow-[-1px_0_10px_rgba(0,0,0,0.02)] z-10 shrink-0">
      <div className="p-5 border-b sticky top-0 bg-white z-10">
        <h3 className="text-base font-semibold text-zinc-900 capitalize object-contain truncate">
          {selectedNode.type} Node
        </h3>
        <p className="text-xs text-zinc-500 mt-0.5 truncate">{selectedNode.id}</p>
      </div>

      <div className="p-5 overflow-y-auto flex flex-col gap-5 flex-1 pb-10">
        <div className="flex flex-col gap-2 relative">
          <label className="text-xs font-semibold text-zinc-700">Label</label>
          <input
            type="text"
            className="text-sm p-2 w-full pt-2 rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
            value={(selectedNode.data.label as string) || ''}
            onChange={(e) => handleChange('label', e.target.value)}
            placeholder="E.g. Get items"
          />
        </div>

        {selectedNode.type === 'step' && (
          <>
            <div className="flex flex-col gap-2 relative">
              <label className="text-xs font-semibold text-zinc-700">Tool Name <span className="text-red-500">*</span></label>
              <input
                type="text"
                className="text-sm p-2 w-full pt-2 rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                value={(selectedNode.data.tool as string) || ''}
                onChange={(e) => handleChange('tool', e.target.value)}
                placeholder="E.g. slack_post_message"
              />
              {!selectedNode.data.tool && (
                <p className="text-[10px] text-red-500 mt-1 absolute -bottom-4">Required field missing</p>
              )}
            </div>
            <div className="flex flex-col gap-2">
              <label className="text-xs font-semibold text-zinc-700">Timeout (ms)</label>
              <input
                type="number"
                className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white"
                value={(selectedNode.data.timeout as string) || '30000'}
                onChange={(e) => handleChange('timeout', e.target.value)}
              />
            </div>
          </>
        )}

        <div className="flex flex-col gap-2">
          <label className="text-xs font-semibold text-zinc-700">Description</label>
          <textarea
            className="text-sm p-2 w-full rounded-md border border-zinc-200 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-zinc-50 focus:bg-white min-h-[80px] resize-y"
            value={(selectedNode.data.description as string) || ''}
            onChange={(e) => handleChange('description', e.target.value)}
            placeholder="Add some notes about this node..."
          />
        </div>
      </div>
    </div>
  );
}

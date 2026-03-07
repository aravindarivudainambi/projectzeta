import React from 'react';
import { Play, Code, GitBranch, ArrowRight, UserCheck } from 'lucide-react';

const nodeTypesList = [
  { type: 'trigger', label: 'Trigger', icon: Play, desc: 'Start workflow', color: 'emerald' },
  { type: 'step', label: 'Tool Step', icon: Code, desc: 'Call an integration', color: 'blue' },
  { type: 'condition', label: 'Condition', icon: GitBranch, desc: 'If/Else logic', color: 'amber' },
  { type: 'human', label: 'Human Approval', icon: UserCheck, desc: 'Wait for review', color: 'purple' },
  { type: 'output', label: 'Output', icon: ArrowRight, desc: 'End workflow', color: 'zinc' },
];

export function NodePalette() {
  const onDragStart = (event: React.DragEvent, nodeType: string, label: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.setData('application/reactflow-label', label);
    event.dataTransfer.effectAllowed = 'move';
  };

  return (
    <div className="w-64 bg-white border-r h-full overflow-y-auto flex flex-col items-stretch pt-4 pb-4 shadow-[1px_0_10px_rgba(0,0,0,0.02)] z-10 shrink-0">
      <div className="px-5 mb-4">
        <h3 className="text-sm font-semibold text-zinc-900 mb-1">Add Nodes</h3>
        <p className="text-xs text-zinc-500">Drag to canvas to add</p>
      </div>
      <div className="px-3 flex flex-col gap-2">
        {nodeTypesList.map((item) => {
          const Icon = item.icon;
          return (
            <div
              key={item.type}
              className={`p-3 border rounded-lg bg-white shadow-sm cursor-grab active:cursor-grabbing hover:bg-zinc-50 transition-colors flex items-center gap-3`}
              onDragStart={(event) => onDragStart(event, item.type, item.label)}
              draggable
            >
              <div className={`w-8 h-8 rounded shrink-0 flex items-center justify-center bg-${item.color}-50 text-${item.color}-600`}>
                <Icon size={16} />
              </div>
              <div className="flex flex-col">
                <span className="text-sm font-medium text-zinc-800">{item.label}</span>
                <span className="text-xs text-zinc-500">{item.desc}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

import React from 'react';
import { Handle, Position } from '@xyflow/react';
import { Play, Code, GitBranch, ArrowRight, UserCheck } from 'lucide-react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const TriggerNode = ({ data, selected }: any) => {
  return (
    <div className={cn("px-4 py-3 shadow-md rounded-lg bg-emerald-50 border-2", selected ? "border-emerald-500" : "border-emerald-200")}>
      <div className="flex items-center gap-2">
        <div className="rounded-full w-8 h-8 flex items-center justify-center bg-emerald-100 text-emerald-600">
          <Play size={16} />
        </div>
        <div>
          <div className="text-sm font-bold text-emerald-900">Trigger</div>
          <div className="text-xs text-emerald-700">{data.label || 'Schedule / Event'}</div>
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} className="w-3 h-3 bg-emerald-500" />
    </div>
  );
};

export const StepNode = ({ data, selected }: any) => {
  return (
    <div className={cn("px-4 py-3 shadow-md rounded-lg bg-blue-50 border-2 min-w-[200px]", selected ? "border-blue-500" : "border-blue-200")}>
      <Handle type="target" position={Position.Top} className="w-3 h-3 bg-blue-500" />
      <div className="flex items-center gap-2">
        <div className="rounded-full w-8 h-8 flex items-center justify-center bg-blue-100 text-blue-600">
          <Code size={16} />
        </div>
        <div>
          <div className="text-sm font-bold text-blue-900">{data.tool || 'Tool Call'}</div>
          <div className="text-xs text-blue-700">{data.label || 'Execute step'}</div>
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} className="w-3 h-3 bg-blue-500" />
    </div>
  );
};

export const ConditionNode = ({ data, selected }: any) => {
  return (
    <div className={cn("px-4 py-3 shadow-md rounded-lg bg-amber-50 border-2", selected ? "border-amber-500" : "border-amber-200")}>
      <Handle type="target" position={Position.Top} className="w-3 h-3 bg-amber-500" />
      <div className="flex items-center gap-2">
        <div className="rounded-full w-8 h-8 flex items-center justify-center bg-amber-100 text-amber-600">
          <GitBranch size={16} />
        </div>
        <div>
          <div className="text-sm font-bold text-amber-900">Condition</div>
          <div className="text-xs text-amber-700">{data.label || 'If / Else'}</div>
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} id="true" style={{ left: '30%' }} className="w-3 h-3 bg-amber-500" />
      <Handle type="source" position={Position.Bottom} id="false" style={{ left: '70%' }} className="w-3 h-3 bg-amber-500" />
    </div>
  );
};

export const OutputNode = ({ data, selected }: any) => {
  return (
    <div className={cn("px-4 py-3 shadow-md rounded-lg bg-zinc-50 border-2", selected ? "border-zinc-500" : "border-zinc-200")}>
      <Handle type="target" position={Position.Top} className="w-3 h-3 bg-zinc-500" />
      <div className="flex items-center gap-2">
        <div className="rounded-full w-8 h-8 flex items-center justify-center bg-zinc-200 text-zinc-600">
          <ArrowRight size={16} />
        </div>
        <div>
          <div className="text-sm font-bold text-zinc-900">Output</div>
          <div className="text-xs text-zinc-700">{data.label || 'End workflow'}</div>
        </div>
      </div>
    </div>
  );
};

export const HumanApprovalNode = ({ data, selected }: any) => {
  return (
    <div className={cn("px-4 py-3 shadow-md rounded-lg bg-purple-50 border-2 min-w-[200px]", selected ? "border-purple-500" : "border-purple-200")}>
      <Handle type="target" position={Position.Top} className="w-3 h-3 bg-purple-500" />
      <div className="flex items-center gap-2">
        <div className="rounded-full w-8 h-8 flex items-center justify-center bg-purple-100 text-purple-600">
          <UserCheck size={16} />
        </div>
        <div>
          <div className="text-sm font-bold text-purple-900">Human Approval</div>
          <div className="text-xs text-purple-700">{data.label || 'Wait for review'}</div>
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} className="w-3 h-3 bg-purple-500" />
    </div>
  );
};

export const nodeTypes = {
  trigger: TriggerNode,
  step: StepNode,
  condition: ConditionNode,
  output: OutputNode,
  human: HumanApprovalNode,
};

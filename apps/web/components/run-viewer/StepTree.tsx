"use client";

import { StepCard, StepData } from "./StepCard";

interface StepTreeProps {
  steps: StepData[];
}

export function StepTree({ steps }: StepTreeProps) {
  if (!steps.length) return null;

  return (
    <div className="flex flex-col pl-2">
      {steps.map((step, index) => (
        <StepCard 
          key={step.id} 
          step={step} 
          isLast={index === steps.length - 1} 
        />
      ))}
    </div>
  );
}

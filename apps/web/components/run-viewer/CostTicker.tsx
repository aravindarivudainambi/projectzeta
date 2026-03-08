"use client";

import { useEffect, useState } from "react";
import { DollarSign } from "lucide-react";
import { motion, useAnimation } from "framer-motion";

interface CostTickerProps {
  cost: number;
}

export function CostTicker({ cost }: CostTickerProps) {
  const [displayCost, setDisplayCost] = useState(cost);
  const controls = useAnimation();

  useEffect(() => {
    // Animate the ticker effect when cost increases
    if (cost > displayCost) {
      controls.start({
        y: [0, -5, 0],
        color: ["#10b981", "#ffffff"],
        transition: { duration: 0.3 },
      });
      setDisplayCost(cost);
    }
  }, [cost, displayCost, controls]);

  return (
    <div className="flex items-center gap-2 rounded-md border border-zinc-200 bg-white px-3 py-2 text-sm font-mono shadow-sm">
      <DollarSign className="w-4 h-4 text-emerald-500" />
      <motion.span
        animate={controls}
        className="font-semibold tracking-wider text-zinc-900"
      >
        {displayCost.toFixed(4)}
      </motion.span>
    </div>
  );
}

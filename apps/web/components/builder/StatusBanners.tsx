import React from 'react';
import { AlertCircle, TerminalSquare, AlertTriangle, RefreshCcw, WifiOff, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';
import { motion, AnimatePresence } from 'framer-motion';

export type StatusBannerType = 'error' | 'warning' | 'info' | 'timeout' | 'disconnected';

interface StatusBannerProps {
  type: StatusBannerType;
  title: string;
  message?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  visible: boolean;
  className?: string;
}

const bannerConfig = {
  error: {
    icon: AlertCircle,
    styles: 'bg-red-500/10 border-red-500/20 text-red-500',
    iconStyles: 'text-red-500',
  },
  warning: {
    icon: AlertTriangle,
    styles: 'bg-amber-500/10 border-amber-500/20 text-amber-500',
    iconStyles: 'text-amber-500',
  },
  info: {
    icon: TerminalSquare,
    styles: 'bg-blue-500/10 border-blue-500/20 text-blue-500',
    iconStyles: 'text-blue-500',
  },
  timeout: {
    icon: Clock,
    styles: 'bg-orange-500/10 border-orange-500/20 text-orange-500',
    iconStyles: 'text-orange-500',
  },
  disconnected: {
    icon: WifiOff,
    styles: 'bg-zinc-500/10 border-zinc-500/20 text-zinc-400',
    iconStyles: 'text-zinc-500',
  }
};

export function StatusBanners({ type, title, message, action, visible, className }: StatusBannerProps) {
  const config = bannerConfig[type];
  const Icon = config.icon;

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95 }}
          className={cn(
            'flex items-center justify-between p-4 rounded-lg border backdrop-blur-sm',
            config.styles,
            className
          )}
        >
          <div className="flex items-start gap-3">
            <Icon className={cn("w-5 h-5 mt-0.5 shrink-0", config.iconStyles)} />
            <div className="flex flex-col">
              <span className="font-semibold text-sm">{title}</span>
              {message && <span className="opacity-80 text-sm mt-0.5">{message}</span>}
            </div>
          </div>
          
          {action && (
            <button
              onClick={action.onClick}
              className="ml-4 shrink-0 flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-white/10 hover:bg-white/20 transition-colors"
            >
              {type === 'error' || type === 'disconnected' || type === 'timeout' ? (
                <RefreshCcw className="w-3.5 h-3.5" />
              ) : null}
              {action.label}
            </button>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}

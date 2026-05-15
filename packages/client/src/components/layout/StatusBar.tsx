import * as React from 'react';
import { cn } from '../../lib/utils';

interface StatusBarProps {
  skillCount?: number;
  connectionStatus?: 'connected' | 'disconnected' | 'checking';
  lastUpdated?: string | null;
  className?: string;
}

const statusConfig = {
  connected: { label: 'Connected', dotClass: 'bg-green-500' },
  disconnected: { label: 'Disconnected', dotClass: 'bg-red-500' },
  checking: { label: 'Checking...', dotClass: 'bg-yellow-500 animate-pulse' },
};

function StatusBar({
  skillCount,
  connectionStatus = 'connected',
  lastUpdated,
  className,
}: StatusBarProps) {
  const status = statusConfig[connectionStatus];

  return (
    <footer
      className={cn(
        'flex items-center justify-between h-7 px-4 border-t border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-900 text-xs text-slate-500 dark:text-slate-400',
        className,
      )}
    >
      <div className="flex items-center gap-4">
        {skillCount !== undefined && (
          <span>{skillCount} skill{skillCount !== 1 ? 's' : ''}</span>
        )}
      </div>
      <div className="flex items-center gap-4">
        {lastUpdated && (
          <span>Last updated: {lastUpdated}</span>
        )}
        <span className="flex items-center gap-1.5">
          <span className={cn('h-2 w-2 rounded-full', status.dotClass)} />
          {status.label}
        </span>
      </div>
    </footer>
  );
}

export { StatusBar };

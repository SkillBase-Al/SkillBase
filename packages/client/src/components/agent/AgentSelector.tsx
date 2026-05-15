import * as React from 'react';
import { Check, ChevronDown } from 'lucide-react';
import { cn } from '../../lib/utils';
import type { AgentConfig } from '../../types/agent';

interface AgentSelectorProps {
  agents: AgentConfig[];
  selectedIds: string[];
  onChange: (ids: string[]) => void;
  placeholder?: string;
  className?: string;
}

function AgentSelector({
  agents,
  selectedIds,
  onChange,
  placeholder = 'Select agents...',
  className,
}: AgentSelectorProps) {
  const [open, setOpen] = React.useState(false);
  const ref = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const selectedLabels = agents
    .filter((a) => selectedIds.includes(a.id))
    .map((a) => a.name);

  const handleToggle = (id: string) => {
    const newSelection = selectedIds.includes(id)
      ? selectedIds.filter((s) => s !== id)
      : [...selectedIds, id];
    onChange(newSelection);
  };

  return (
    <div ref={ref} className={cn('relative', className)}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="flex items-center justify-between w-full h-10 rounded-md border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-900 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <span
          className={cn(
            'truncate',
            selectedLabels.length === 0
              ? 'text-slate-400'
              : 'text-slate-800 dark:text-slate-200',
          )}
        >
          {selectedLabels.length > 0
            ? selectedLabels.join(', ')
            : placeholder}
        </span>
        <ChevronDown className="h-4 w-4 text-slate-400 flex-shrink-0 ml-2" />
      </button>

      {open && (
        <div className="absolute z-10 mt-1 w-full rounded-md border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 shadow-lg max-h-60 overflow-auto">
          {agents.length === 0 ? (
            <div className="px-3 py-2 text-sm text-slate-400">
              No agents configured
            </div>
          ) : (
            agents.map((agent) => (
              <button
                key={agent.id}
                type="button"
                onClick={() => handleToggle(agent.id)}
                className="flex items-center gap-2 w-full px-3 py-2 text-sm hover:bg-slate-100 dark:hover:bg-slate-800"
              >
                <div
                  className={cn(
                    'h-4 w-4 rounded border flex items-center justify-center transition-colors',
                    selectedIds.includes(agent.id)
                      ? 'border-blue-600 bg-blue-600'
                      : 'border-slate-300 dark:border-slate-600',
                  )}
                >
                  {selectedIds.includes(agent.id) && (
                    <Check className="h-3 w-3 text-white" />
                  )}
                </div>
                <div className="text-left">
                  <p className="text-sm font-medium text-slate-800 dark:text-slate-200">
                    {agent.name}
                  </p>
                  <p className="text-xs text-slate-400">{agent.agentType}</p>
                </div>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}

export { AgentSelector };

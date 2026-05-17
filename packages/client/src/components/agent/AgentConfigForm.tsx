import * as React from 'react';
import { FolderOpen } from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { cn } from '../../lib/utils';
import type { AgentConfig } from '../../types/agent';

interface AgentConfigFormProps {
  initialData?: Partial<AgentConfig>;
  onSubmit: (data: {
    name: string;
    type: string;
    path: string;
  }) => void;
  onCancel?: () => void;
  isSubmitting?: boolean;
  className?: string;
}

const AGENT_TYPES: Array<{ value: string; label: string }> = [
  { value: 'cursor', label: 'Cursor' },
  { value: 'claude', label: 'Claude Code' },
  { value: 'codex', label: 'Codex' },
  { value: 'windsurf', label: 'Windsurf' },
  { value: 'qoder', label: 'Qoder' },
  { value: 'opencode', label: 'OpenCode' },
  { value: 'custom', label: 'Custom' },
];

const AGENT_DEFAULT_PATHS: Record<string, string> = {
  cursor: '~/.cursor/skills/',
  claude: '~/.claude/skills/',
  codex: '~/.codex/skills/',
  windsurf: '~/.codeium/windsurf/global_workflows/',
  qoder: '~/.qoder/skills;~/.agents/skills',
  opencode: '~/.config/opencode/skills/',
};

function AgentConfigForm({
  initialData,
  onSubmit,
  onCancel,
  isSubmitting = false,
  className,
}: AgentConfigFormProps) {
  const [name, setName] = React.useState(initialData?.name ?? '');
  const [type, setType] = React.useState(
    initialData?.agentType ?? 'custom',
  );
  const [path, setPath] = React.useState(initialData?.basePath ?? '');
  const lastAutoPath = React.useRef('');

  // Auto-fill path when type changes (unless manually edited)
  const handleTypeChange = (newType: string) => {
    setType((prevType) => {
      if (prevType === newType) return newType;
      const defaultPath = AGENT_DEFAULT_PATHS[newType] ?? '';
      const prevDefault = AGENT_DEFAULT_PATHS[prevType] ?? '';
      // Update path if empty, matches previous default, or matches last auto-fill
      if (
        !path ||
        path === lastAutoPath.current ||
        (prevDefault && path === prevDefault)
      ) {
        setPath(defaultPath);
        lastAutoPath.current = defaultPath;
      }
      return newType;
    });
  };

  const handlePathChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setPath(e.target.value);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !path.trim()) return;
    onSubmit({ name: name.trim(), type, path: path.trim() });
  };

  const isValid = name.trim().length > 0 && path.trim().length > 0;

  return (
    <form
      onSubmit={handleSubmit}
      className={cn('space-y-4', className)}
    >
      {/* Name */}
      <div className="space-y-1.5">
        <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
          Agent Name
        </label>
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My Agent"
          required
        />
      </div>

      {/* Type */}
      <div className="space-y-1.5">
        <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
          Agent Type
        </label>
        <div className="flex gap-2">
          {AGENT_TYPES.map((option) => (
            <button
              key={option.value}
              type="button"
              onClick={() => handleTypeChange(option.value)}
              className={cn(
                'flex-1 px-3 py-2 rounded-md border text-sm font-medium transition-colors',
                type === option.value
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300'
                  : 'border-slate-300 dark:border-slate-600 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800',
              )}
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>

      {/* Path */}
      <div className="space-y-1.5">
        <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
          Installation Path
        </label>
        <div className="flex gap-2">
          <Input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            placeholder="/path/to/skills"
            className="flex-1"
            required
          />
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={() => {
              // In Tauri this would invoke a dialog folder picker
              // Placeholder for now
            }}
            title="Browse for folder"
          >
            <FolderOpen className="h-4 w-4" />
          </Button>
        </div>
        <p className="text-xs text-slate-400">
          Separate multiple paths with <span className="font-mono">;</span>
        </p>
      </div>

      {/* Actions */}
      <div className="flex items-center justify-end gap-2 pt-2">
        {onCancel && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={onCancel}
            disabled={isSubmitting}
          >
            Cancel
          </Button>
        )}
        <Button type="submit" size="sm" disabled={!isValid || isSubmitting}>
          {isSubmitting
            ? 'Saving...'
            : initialData
              ? 'Update Agent'
              : 'Add Agent'}
        </Button>
      </div>
    </form>
  );
}

export { AgentConfigForm };

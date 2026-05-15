import * as React from 'react';
import { AlertCircle, AlertTriangle, Info } from 'lucide-react';
import { cn } from '../../lib/utils';
import type { Issue } from '../../types/assessment';

interface IssueListProps {
  issues: Issue[];
  className?: string;
}

const severityConfig = {
  error: { icon: AlertCircle, color: 'text-red-500', bg: 'bg-red-50 dark:bg-red-950/50', border: 'border-red-200 dark:border-red-800' },
  warning: { icon: AlertTriangle, color: 'text-yellow-500', bg: 'bg-yellow-50 dark:bg-yellow-950/50', border: 'border-yellow-200 dark:border-yellow-800' },
  info: { icon: Info, color: 'text-blue-500', bg: 'bg-blue-50 dark:bg-blue-950/50', border: 'border-blue-200 dark:border-blue-800' },
};

function IssueList({ issues, className }: IssueListProps) {
  if (issues.length === 0) {
    return (
      <div className={cn('text-center py-6', className)}>
        <p className="text-sm text-slate-400">No issues found</p>
      </div>
    );
  }

  return (
    <div className={cn('space-y-2', className)}>
      {issues.map((issue, index) => {
        const config = severityConfig[issue.severity];
        const Icon = config.icon;

        return (
          <div
            key={index}
            className={cn(
              'flex items-start gap-3 rounded-lg border p-3',
              config.bg,
              config.border,
            )}
          >
            <Icon className={cn('h-4 w-4 mt-0.5 flex-shrink-0', config.color)} />
            <div className="flex-1 min-w-0">
              <p className="text-sm text-slate-700 dark:text-slate-300">
                {issue.message}
              </p>
              {(issue.line > 0 || issue.code) && (
                <p className="text-xs text-slate-500 mt-0.5">
                  {issue.line > 0 && `Line ${issue.line}${issue.column > 0 ? `:${issue.column}` : ''}`}
                  {issue.line > 0 && issue.code && ' | '}
                  {issue.code && <code className="font-mono">{issue.code}</code>}
                </p>
              )}
            </div>
            <span
              className={cn(
                'text-[10px] font-medium uppercase px-1.5 py-0.5 rounded',
                config.color,
                config.bg,
              )}
            >
              {issue.severity}
            </span>
          </div>
        );
      })}
    </div>
  );
}

export { IssueList };

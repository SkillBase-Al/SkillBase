import * as React from 'react';
import { X, FileText } from 'lucide-react';
import { Dialog, DialogHeader, DialogTitle } from '../ui/dialog';
import { Badge } from '../ui/badge';
import { cn } from '../../lib/utils';
import type { DedupGroup } from '../../types/dedup';

interface ComparisonModalProps {
  group: DedupGroup | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function ComparisonModal({ group, open, onOpenChange }: ComparisonModalProps) {
  if (!group) return null;

  const similarityPercent = (group.similarity * 100).toFixed(0);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogHeader>
        <div className="flex items-center justify-between">
          <DialogTitle>Skill Comparison</DialogTitle>
          <Badge
            variant={
              group.similarity >= 0.9
                ? 'danger'
                : group.similarity >= 0.7
                  ? 'warning'
                  : 'info'
            }
          >
            {similarityPercent}% match
          </Badge>
        </div>
      </DialogHeader>

      <div className="space-y-4">
        {/* Side by Side */}
        <div className="grid grid-cols-2 gap-4">
          {group.skills.map((skill, index) => (
            <div
              key={skill.id}
              className="space-y-2"
            >
              <div
                className={cn(
                  'flex items-center gap-2 p-2 rounded-t-lg border-b-2',
                  index === 0
                    ? 'bg-blue-50 dark:bg-blue-950 border-blue-400'
                    : 'bg-purple-50 dark:bg-purple-950 border-purple-400',
                )}
              >
                <FileText className={cn(
                  'h-4 w-4',
                  index === 0 ? 'text-blue-500' : 'text-purple-500',
                )} />
                <span className={cn(
                  'text-sm font-medium',
                  index === 0 ? 'text-blue-700 dark:text-blue-300' : 'text-purple-700 dark:text-purple-300',
                )}>
                  {skill.name}
                </span>
              </div>

              <div className="rounded-b-lg border border-slate-200 dark:border-slate-700 p-3 bg-white dark:bg-slate-900 space-y-2">
                <div>
                  <span className="text-[10px] font-medium text-slate-400 uppercase tracking-wider">
                    Description
                  </span>
                  <p className="text-xs text-slate-600 dark:text-slate-400 mt-0.5">
                    {skill.description}
                  </p>
                </div>
                <div>
                  <span className="text-[10px] font-medium text-slate-400 uppercase tracking-wider">
                    Version
                  </span>
                  <p className="text-xs text-slate-600 dark:text-slate-400 mt-0.5">
                    {skill.version ?? 'N/A'}
                  </p>
                </div>
                <div>
                  <span className="text-[10px] font-medium text-slate-400 uppercase tracking-wider">
                    Path
                  </span>
                  <p className="text-xs text-slate-600 dark:text-slate-400 mt-0.5 break-all font-mono">
                    {skill.path}
                  </p>
                </div>
                <div>
                  <span className="text-[10px] font-medium text-slate-400 uppercase tracking-wider">
                    ID
                  </span>
                  <p className="text-xs text-slate-600 dark:text-slate-400 mt-0.5 font-mono">
                    {skill.id}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </Dialog>
  );
}

export { ComparisonModal };

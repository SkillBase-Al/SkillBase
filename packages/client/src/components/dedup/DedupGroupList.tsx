import * as React from 'react';
import { DedupGroupCard } from './DedupGroupCard';
import { LoadingSkeleton } from '../shared/LoadingSkeleton';
import { EmptyState } from '../shared/EmptyState';
import { ErrorState } from '../shared/ErrorState';
import { GitCompareArrows, SearchCheck } from 'lucide-react';
import type { DedupGroup } from '../../types/dedup';

interface DedupGroupListProps {
  groups: DedupGroup[];
  isLoading: boolean;
  error: string | null;
  onRetry?: () => void;
  onDelete: (groupId: string, skillId: string) => void;
  onCompare: (group: DedupGroup) => void;
}

function DedupGroupList({
  groups,
  isLoading,
  error,
  onRetry,
  onDelete,
  onCompare,
}: DedupGroupListProps) {
  if (isLoading) {
    return <LoadingSkeleton variant="list" count={4} />;
  }

  if (error) {
    return <ErrorState message={error} onRetry={onRetry} />;
  }

  if (groups.length === 0) {
    return (
      <EmptyState
        icon={<SearchCheck className="h-12 w-12" />}
        title="No duplicates found"
        description="All your skills appear to be unique. Run a scan to check for duplicates."
      />
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <GitCompareArrows className="h-5 w-5 text-slate-500" />
          <span className="text-sm text-slate-600 dark:text-slate-400">
            Found {groups.length} duplicate group{groups.length !== 1 ? 's' : ''}
          </span>
        </div>
      </div>
      {groups
        .sort((a, b) => b.similarity - a.similarity)
        .map((group) => (
          <DedupGroupCard
            key={group.id}
            group={group}
            onDelete={(skillId) => onDelete(group.id, skillId)}
            onCompare={() => onCompare(group)}
          />
        ))}
    </div>
  );
}

export { DedupGroupList };

import * as React from 'react';
import { GitCompareArrows, ScanLine, SearchCheck, Loader2 } from 'lucide-react';
import { Button } from '../components/ui/button';
import { DedupGroupList } from '../components/dedup/DedupGroupList';
import { ComparisonModal } from '../components/dedup/ComparisonModal';
import { useDedupStore } from '../stores/dedupStore';
import type { DedupGroup } from '../types/dedup';

function DedupPage() {
  const { groups, isLoading, error, runDedup, deleteSkill } = useDedupStore();
  const [compareGroup, setCompareGroup] = React.useState<DedupGroup | null>(null);
  const [compareOpen, setCompareOpen] = React.useState(false);

  React.useEffect(() => {
    runDedup();
  }, []);

  const handleDelete = async (groupId: string, skillId: string) => {
    try {
      await deleteSkill(groupId, skillId);
    } catch {
      // Error handled by store
    }
  };

  const handleCompare = (group: DedupGroup) => {
    setCompareGroup(group);
    setCompareOpen(true);
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <GitCompareArrows className="h-5 w-5 text-blue-600" />
            <span className="text-sm font-semibold text-slate-700 dark:text-slate-300">
              Duplicate Detection
            </span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => runDedup()}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                Scanning...
              </>
            ) : (
              <>
                <ScanLine className="h-4 w-4 mr-1" />
                Check Duplicates
              </>
            )}
          </Button>
        </div>

        {/* Progress indicator during scan */}
        {isLoading && groups.length === 0 && (
          <div className="flex flex-col items-center justify-center py-16">
            <Loader2 className="h-10 w-10 text-blue-500 animate-spin mb-4" />
            <p className="text-sm text-slate-500 dark:text-slate-400">
              Scanning for duplicate skills...
            </p>
            <div className="w-64 h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full mt-4 overflow-hidden">
              <div className="h-full bg-blue-500 rounded-full animate-pulse w-2/3" />
            </div>
          </div>
        )}

        {/* Pre-scan empty state */}
        {!isLoading && groups.length === 0 && !error && (
          <div className="flex flex-col items-center justify-center py-16">
            <SearchCheck className="h-12 w-12 text-slate-400 mb-4" />
            <h3 className="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-1">
              No duplicates found
            </h3>
            <p className="text-sm text-slate-500 dark:text-slate-400 max-w-sm text-center mb-6">
              No duplicate skills detected. Your skills collection appears to be
              clean and well-organized.
            </p>
          </div>
        )}

        {/* Error state */}
        {error && (
          <div className="flex flex-col items-center justify-center py-16">
            <div className="rounded-lg border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-950 p-4 max-w-md">
              <p className="text-sm text-red-600 dark:text-red-400 mb-2">
                {error}
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => runDedup()}
              >
                Try Again
              </Button>
            </div>
          </div>
        )}

        {/* Results */}
        {!isLoading && groups.length > 0 && (
          <DedupGroupList
            groups={groups}
            isLoading={false}
            error={null}
            onDelete={handleDelete}
            onCompare={handleCompare}
          />
        )}
      </div>

      {/* Comparison Modal */}
      <ComparisonModal
        group={compareGroup}
        open={compareOpen}
        onOpenChange={setCompareOpen}
      />
    </div>
  );
}

export { DedupPage };

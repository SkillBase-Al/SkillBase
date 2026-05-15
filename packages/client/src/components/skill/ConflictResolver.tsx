import * as React from 'react';
import { AlertTriangle, Check, ChevronRight, FileText, Layers, User } from 'lucide-react';
import { Dialog, DialogHeader, DialogTitle, DialogDescription } from '../ui/dialog';
import { Button } from '../ui/button';
import { getSkillConflicts, resolveSkillConflict } from '../../services/tauri';
import type { SkillConflict, ConflictCandidate } from '../../types/conflict';

interface ConflictResolverProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAllResolved: () => void;
}

function ConflictResolver({ open, onOpenChange, onAllResolved }: ConflictResolverProps) {
  const [conflicts, setConflicts] = React.useState<SkillConflict[]>([]);
  const [currentIndex, setCurrentIndex] = React.useState(0);
  const [selectedCandidate, setSelectedCandidate] = React.useState<string | null>(null);
  const [resolving, setResolving] = React.useState(false);
  const [resolvedIds, setResolvedIds] = React.useState<Set<string>>(new Set());

  const currentConflict = conflicts[currentIndex];
  const totalUnresolved = conflicts.length;

  // Fetch conflicts when dialog opens
  React.useEffect(() => {
    if (open) {
      getSkillConflicts()
        .then((result) => {
          setConflicts(result);
          setCurrentIndex(0);
          setSelectedCandidate(null);
          setResolvedIds(new Set());
        })
        .catch(() => setConflicts([]));
    }
  }, [open]);

  const handleResolve = async () => {
    if (!currentConflict || !selectedCandidate) return;

    setResolving(true);
    try {
      await resolveSkillConflict(currentConflict.id, selectedCandidate);
      setResolvedIds((prev) => new Set(prev).add(currentConflict.id));

      if (currentIndex < conflicts.length - 1) {
        // Advance to next conflict
        setCurrentIndex((i) => i + 1);
        setSelectedCandidate(null);
      } else {
        // All resolved
        onAllResolved();
        onOpenChange(false);
      }
    } catch (e) {
      console.error('Failed to resolve conflict:', e);
    } finally {
      setResolving(false);
    }
  };

  if (!open || conflicts.length === 0) return null;

  const remaining = conflicts.length - resolvedIds.size;
  if (remaining === 0) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogHeader>
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-10 w-10 rounded-full bg-amber-100 dark:bg-amber-900">
            <AlertTriangle className="h-5 w-5 text-amber-600 dark:text-amber-300" />
          </div>
          <div>
            <DialogTitle>Skill Conflict Detected</DialogTitle>
            <DialogDescription>
              The same skill name was found with different content across agents.
              Choose which version to keep — it will be applied to all agents.
            </DialogDescription>
          </div>
        </div>
      </DialogHeader>

      {/* Progress */}
      <div className="flex items-center gap-2 mb-4 text-sm text-slate-500">
        <Layers className="h-4 w-4" />
        <span>
          Conflict {currentIndex + 1} of {totalUnresolved}
        </span>
      </div>

      {/* Conflict header */}
      <div className="rounded-lg border border-amber-200 dark:border-amber-800 bg-amber-50 dark:bg-amber-950 p-3 mb-4">
        <div className="flex items-center gap-2 text-amber-700 dark:text-amber-300 font-medium text-sm">
          <FileText className="h-4 w-4" />
          <span>{currentConflict.skillName}</span>
        </div>
        <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
          This skill has {currentConflict.candidates.length} different versions. Select the one you want to keep.
        </p>
      </div>

      {/* Candidate cards */}
      <div className="space-y-2 mb-4">
        {currentConflict.candidates.map((candidate) => {
          const isSelected = selectedCandidate === candidate.id;
          const shortHash = candidate.contentHash.slice(0, 8);

          return (
            <button
              key={candidate.id}
              onClick={() => setSelectedCandidate(candidate.id)}
              className={`w-full text-left rounded-lg border p-3 transition-colors ${
                isSelected
                  ? 'border-blue-400 dark:border-blue-500 bg-blue-50 dark:bg-blue-950 ring-1 ring-blue-400'
                  : 'border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 hover:border-slate-300 dark:hover:border-slate-600'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 min-w-0 flex-1">
                  <User className="h-4 w-4 text-slate-400 flex-shrink-0" />
                  <span className="text-sm font-medium text-slate-800 dark:text-slate-200 truncate">
                    {candidate.agentName}
                  </span>
                  <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 flex-shrink-0">
                    {candidate.agentType}
                  </span>
                </div>
                <div className="flex items-center gap-2 flex-shrink-0 ml-2">
                  {candidate.version && (
                    <span className="text-xs text-slate-500">v{candidate.version}</span>
                  )}
                  <span className="text-[10px] font-mono text-slate-400">{shortHash}</span>
                  {isSelected && (
                    <div className="flex items-center justify-center h-5 w-5 rounded-full bg-blue-500">
                      <Check className="h-3 w-3 text-white" />
                    </div>
                  )}
                  {!isSelected && (
                    <div className="h-5 w-5 rounded-full border-2 border-slate-300 dark:border-slate-600" />
                  )}
                </div>
              </div>
              <p className="text-xs text-slate-400 mt-1 truncate">{candidate.skillDir}</p>
            </button>
          );
        })}
      </div>

      {/* Actions */}
      <div className="flex items-center justify-between pt-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => onOpenChange(false)}
        >
          Later
        </Button>
        <Button
          size="sm"
          onClick={handleResolve}
          disabled={!selectedCandidate || resolving}
        >
          {resolving ? 'Resolving...' : 'Keep Selected Version'}
          <ChevronRight className="h-4 w-4 ml-1" />
        </Button>
      </div>
    </Dialog>
  );
}

export { ConflictResolver };

import * as React from 'react';
import {
  Package,
  User,
  Tag,
  Shield,
  FileText,
  X,
  Download,
  Trash2,
  RefreshCw,
  ChevronDown,
  ChevronUp,
  FileCode,
} from 'lucide-react';
import { Dialog, DialogHeader, DialogTitle, DialogDescription } from '../ui/dialog';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Switch } from '../ui/switch';
import { cn } from '../../lib/utils';
import { getSkillFileContent } from '../../services/tauri';
import type { InstalledSkill, MarketSkill } from '../../types/skill';
import type { AgentConfig } from '../../types/agent';

type DetailSkill = InstalledSkill | MarketSkill;

interface SkillDetailProps {
  skill: DetailSkill | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onInstall?: () => void;
  onUninstall?: () => void;
  onUpdate?: () => void;
  isInstalled?: boolean;
  agentsCount?: number;
  agentNames?: string[];
  agents?: AgentConfig[];
  onApplyToAgent?: (agentId: string) => Promise<void>;
  onRemoveFromAgent?: (agentId: string) => Promise<void>;
}

function isInstalledSkill(skill: DetailSkill): skill is InstalledSkill {
  return 'enabled' in skill;
}

function isMarketSkill(skill: DetailSkill): skill is MarketSkill {
  return 'category' in skill;
}

function SkillDetail({
  skill,
  open,
  onOpenChange,
  onInstall,
  onUninstall,
  onUpdate,
  isInstalled,
  agentsCount,
  agentNames,
  agents,
  onApplyToAgent,
  onRemoveFromAgent,
}: SkillDetailProps) {
  const [togglingAgents, setTogglingAgents] = React.useState<Set<string>>(new Set());
  const [showUninstallConfirm, setShowUninstallConfirm] = React.useState(false);
  const [showSkillContent, setShowSkillContent] = React.useState(false);
  const [skillContent, setSkillContent] = React.useState<string | null>(null);
  const [loadingContent, setLoadingContent] = React.useState(false);

  // Reset content state when switching skills or closing
  React.useEffect(() => {
    setShowSkillContent(false);
    setSkillContent(null);
  }, [skill?.id, open]);

  if (!skill) return null;

  const handleAgentToggle = async (agentId: string, isApplied: boolean) => {
    setTogglingAgents((prev) => new Set(prev).add(agentId));
    try {
      if (isApplied) {
        await onRemoveFromAgent?.(agentId);
      } else {
        await onApplyToAgent?.(agentId);
      }
    } finally {
      setTogglingAgents((prev) => {
        const next = new Set(prev);
        next.delete(agentId);
        return next;
      });
    }
  };

  const handleConfirmUninstall = () => {
    setShowUninstallConfirm(false);
    onUninstall?.();
  };

  const handleViewContent = async () => {
    if (showSkillContent) {
      setShowSkillContent(false);
      return;
    }

    // If content is already available on the skill object, show it directly
    if (skill.skillContent) {
      setSkillContent(skill.skillContent);
      setShowSkillContent(true);
      return;
    }

    // For installed skills, fetch content from disk
    if (isInstalled && 'id' in skill) {
      setLoadingContent(true);
      try {
        const content = await getSkillFileContent((skill as InstalledSkill).id);
        setSkillContent(content);
        setShowSkillContent(true);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setSkillContent(`<!-- Failed to read SKILL.md: ${msg} -->`);
        setShowSkillContent(true);
      } finally {
        setLoadingContent(false);
      }
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center h-10 w-10 rounded-lg bg-blue-100 dark:bg-blue-900">
              <Package className="h-5 w-5 text-blue-600 dark:text-blue-300" />
            </div>
            <div>
              <DialogTitle>{skill.name}</DialogTitle>
              <DialogDescription>
                {skill.version ? `v${skill.version}` : ''} {skill.author && `by ${skill.author}`}
              </DialogDescription>
            </div>
          </div>
          <button
            onClick={() => onOpenChange(false)}
            className="text-slate-400 hover:text-slate-600"
          >
            <X className="h-5 w-5" />
          </button>
        </div>
      </DialogHeader>

      {showSkillContent && skillContent !== null ? (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300">
              SKILL.md
            </h4>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowSkillContent(false)}
            >
              <ChevronUp className="h-4 w-4 mr-1" />
              Back to details
            </Button>
          </div>
          <pre className="text-xs leading-relaxed bg-slate-50 dark:bg-slate-950 border border-slate-200 dark:border-slate-700 rounded-lg p-4 overflow-auto max-h-[60vh] whitespace-pre-wrap break-words font-mono text-slate-700 dark:text-slate-300">
            {skillContent}
          </pre>
        </div>
      ) : (
        <div className="space-y-4 relative">
          {/* Tags */}
          {skill.tags.length > 0 && (
            <div className="flex flex-wrap gap-1.5">
              {skill.tags.map((tag) => (
                <Badge key={tag} variant="outline" className="text-xs">
                  <Tag className="h-3 w-3 mr-1" />
                  {tag}
                </Badge>
              ))}
            </div>
          )}

          {/* Description */}
          <div>
            <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
              Description
            </h4>
            <p className="text-sm text-slate-500 dark:text-slate-400">
              {skill.description}
            </p>
          </div>

          {/* Assessment Scores (installed) */}
          {isInstalledSkill(skill) && (
            <div>
              <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
                Assessment Scores
              </h4>
              <div className="grid grid-cols-2 gap-3">
                <div
                  className={cn(
                    'rounded-lg border p-3 text-center',
                    skill.formatScore !== undefined && skill.formatScore >= 80
                      ? 'border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-950'
                      : skill.formatScore !== undefined && skill.formatScore >= 50
                        ? 'border-yellow-200 dark:border-yellow-800 bg-yellow-50 dark:bg-yellow-950'
                        : 'border-slate-200 dark:border-slate-700',
                  )}
                >
                  <div className="flex items-center justify-center gap-1 mb-1">
                    <FileText className="h-4 w-4 text-slate-500" />
                    <span className="text-xs text-slate-500">Format</span>
                  </div>
                  <span
                    className={cn(
                      'text-lg font-bold',
                      skill.formatScore !== undefined && skill.formatScore >= 80
                        ? 'text-green-600 dark:text-green-400'
                        : skill.formatScore !== undefined && skill.formatScore >= 50
                          ? 'text-yellow-600 dark:text-yellow-400'
                          : 'text-slate-600 dark:text-slate-400',
                    )}
                  >
                    {skill.formatScore ?? 'N/A'}
                  </span>
                </div>
                <div
                  className={cn(
                    'rounded-lg border p-3 text-center',
                    skill.safetyLevel === 'Safe'
                      ? 'border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-950'
                      : skill.safetyLevel === 'Warning'
                        ? 'border-yellow-200 dark:border-yellow-800 bg-yellow-50 dark:bg-yellow-950'
                        : 'border-slate-200 dark:border-slate-700',
                  )}
                >
                  <div className="flex items-center justify-center gap-1 mb-1">
                    <Shield className="h-4 w-4 text-slate-500" />
                    <span className="text-xs text-slate-500">Security</span>
                  </div>
                  <span
                    className={cn(
                      'text-lg font-bold',
                      skill.safetyLevel === 'Safe'
                        ? 'text-green-600 dark:text-green-400'
                        : skill.safetyLevel === 'Warning'
                          ? 'text-yellow-600 dark:text-yellow-400'
                          : 'text-slate-600 dark:text-slate-400',
                    )}
                  >
                    {skill.safetyLevel ?? 'N/A'}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Agent list (installed) */}
          {isInstalled && agents && agents.length > 0 && isInstalledSkill(skill) && (
            <div>
              <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
                Applied to Agents
              </h4>
              <div className="space-y-1">
                {agents.map((agent) => {
                  const isApplied = skill.agentIds.includes(agent.id);
                  const isToggling = togglingAgents.has(agent.id);
                  return (
                    <div
                      key={agent.id}
                      className="flex items-center justify-between py-2 px-3 rounded-md border border-slate-200 dark:border-slate-700"
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <User className="h-4 w-4 text-slate-400 flex-shrink-0" />
                        <span className="text-sm text-slate-700 dark:text-slate-300 truncate">
                          {agent.name}
                        </span>
                        <Badge variant="outline" className="text-[10px] flex-shrink-0">
                          {agent.agentType}
                        </Badge>
                      </div>
                      <Switch
                        checked={isApplied}
                        disabled={isToggling}
                        onCheckedChange={() => handleAgentToggle(agent.id, isApplied)}
                      />
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Metadata */}
          <div className="flex items-center gap-3 text-sm">
            {isInstalled && isInstalledSkill(skill) && (
              <div className="flex items-center gap-2 text-slate-500">
                <Package className="h-4 w-4" />
                <span>{skill.agentCount} agent(s)</span>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-2 mt-6 pt-4 border-t border-slate-200 dark:border-slate-700">
        {!showSkillContent && (
          <>
            {!isInstalled && onInstall && (
              <Button onClick={onInstall} size="sm" className="flex-1">
                <Download className="h-4 w-4 mr-1" />
                Install
              </Button>
            )}
            {isInstalled && onUninstall && (
              <Button
                onClick={() => setShowUninstallConfirm(true)}
                variant="destructive"
                size="sm"
                className="flex-1"
              >
                <Trash2 className="h-4 w-4 mr-1" />
                Uninstall
              </Button>
            )}
            {isInstalled && onUpdate && (
              <Button onClick={onUpdate} variant="outline" size="sm">
                <RefreshCw className="h-4 w-4 mr-1" />
                Update
              </Button>
            )}
            <Button variant="outline" size="sm" onClick={handleViewContent} disabled={loadingContent}>
              {loadingContent ? (
                <RefreshCw className="h-4 w-4 mr-1 animate-spin" />
              ) : (
                <FileCode className="h-4 w-4 mr-1" />
              )}
              SKILL.md
            </Button>
          </>
        )}
      </div>

      {/* Uninstall confirmation */}
      {showUninstallConfirm && (
        <div className="absolute inset-0 z-10 bg-white/95 dark:bg-slate-900/95 backdrop-blur-sm rounded-lg flex flex-col items-center justify-center p-6">
          <Trash2 className="h-8 w-8 text-red-500 mb-3" />
          <p className="text-sm font-medium text-slate-800 dark:text-slate-200 text-center mb-1">
            Uninstall skill completely?
          </p>
          <p className="text-xs text-slate-500 text-center mb-4">
            This will remove the skill from{' '}
            <span className="font-semibold">all {agentsCount} agent(s)</span>{' '}
            and delete the local files.
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowUninstallConfirm(false)}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={handleConfirmUninstall}
            >
              <Trash2 className="h-4 w-4 mr-1" />
              Confirm Uninstall
            </Button>
          </div>
        </div>
      )}
    </Dialog>
  );
}

export { SkillDetail };

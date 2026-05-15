import * as React from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Package,
  LayoutGrid,
  List,
  Shield,
  ShieldAlert,
  ShieldCheck,
  RefreshCw,
  FileSearch,
  Layers,
} from 'lucide-react';
import { TopBar } from '../components/layout/TopBar';
import { SkillGrid } from '../components/skill/SkillGrid';
import { SkillCard } from '../components/skill/SkillCard';
import { SkillDetail } from '../components/skill/SkillDetail';
import { Button } from '../components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '../components/ui/tabs';
import { Badge } from '../components/ui/badge';
import { LoadingSkeleton } from '../components/shared/LoadingSkeleton';
import { EmptyState } from '../components/shared/EmptyState';
import { useInstalledStore } from '../stores/installedStore';
import { useAssessmentStore } from '../stores/assessmentStore';
import { useAgentStore } from '../stores/agentStore';
import { applySkillToAgents, removeSkillFromAgents, getSkillConflicts } from '../services/tauri';
import { ConflictResolver } from '../components/skill/ConflictResolver';
import type { InstalledSkill } from '../types/skill';

type ViewMode = 'grid' | 'list';
type SafetyFilter = 'all' | 'safe' | 'warning' | 'danger';

function InstalledPage() {
  const navigate = useNavigate();
  const { items, isLoading, error, isEmpty, fetchItems, toggle, uninstall } =
    useInstalledStore();
  const { batchAssess, summary, isLoading: assessLoading } =
    useAssessmentStore();
  const { agents, fetchAgents } = useAgentStore();

  const [searchQuery, setSearchQuery] = React.useState('');
  const [viewMode, setViewMode] = React.useState<ViewMode>('grid');
  const [safetyFilter, setSafetyFilter] = React.useState<SafetyFilter>('all');
  const [agentFilter, setAgentFilter] = React.useState<string>('');
  const [selectedSkill, setSelectedSkill] = React.useState<InstalledSkill | null>(null);
  const [detailOpen, setDetailOpen] = React.useState(false);
  const [conflictOpen, setConflictOpen] = React.useState(false);

  React.useEffect(() => {
    fetchItems();
    fetchAgents();
  }, [fetchItems, fetchAgents]);

  // Check for skill conflicts after items are loaded
  React.useEffect(() => {
    if (!isLoading && items.length > 0) {
      getSkillConflicts()
        .then((conflicts) => {
          if (conflicts.length > 0) {
            setConflictOpen(true);
          }
        })
        .catch(() => {});
    }
  }, [isLoading, items.length]);

  const filteredSkills = React.useMemo(() => {
    let result = items;

    // Search filter
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          s.description.toLowerCase().includes(q) ||
          s.tags.some((t) => t.toLowerCase().includes(q)),
      );
    }

    // Safety filter
    if (safetyFilter === 'safe') {
      result = result.filter(
        (s) => s.safetyLevel === 'Safe',
      );
    } else if (safetyFilter === 'warning') {
      result = result.filter(
        (s) => s.safetyLevel === 'Warning',
      );
    } else if (safetyFilter === 'danger') {
      result = result.filter(
        (s) => s.safetyLevel === 'Dangerous' || s.hasSecurityIssues,
      );
    }

    // Agent filter
    if (agentFilter === '__none__') {
      result = result.filter((s) => s.agentCount === 0);
    } else if (agentFilter) {
      result = result.filter((s) => s.agentIds.includes(agentFilter));
    }

    return result;
  }, [items, searchQuery, safetyFilter, agentFilter]);

  const handleToggle = async (skillId: string, enabled: boolean) => {
    try {
      await toggle(skillId, enabled);
    } catch {
      // Error handled by store
    }
  };

  const handleBatchAssess = async () => {
    if (items.length === 0) return;
    try {
      await batchAssess();
      await fetchItems(); // Refresh items so cards show updated safetyLevel/formatScore
    } catch {
      // Error handled by store
    }
  };

  const handleUninstall = async (skillId: string) => {
    try {
      await uninstall(skillId);
      setDetailOpen(false);
    } catch {
      // Error handled by store
    }
  };

  const safetyButtonIcon = (filter: SafetyFilter) => {
    switch (filter) {
      case 'safe':
        return <ShieldCheck className="h-4 w-4" />;
      case 'warning':
        return <ShieldAlert className="h-4 w-4" />;
      case 'danger':
        return <ShieldAlert className="h-4 w-4" />;
      default:
        return <Shield className="h-4 w-4" />;
    }
  };

  const agentNameMap = React.useMemo(() => {
    const map: Record<string, string> = {};
    for (const a of agents) map[a.id] = a.name;
    return map;
  }, [agents]);

  const handleApplyToAgent = async (agentId: string) => {
    if (!selectedSkill) return;
    await applySkillToAgents(selectedSkill.id, [agentId]);
    await fetchItems();
    const updated = useInstalledStore.getState().items.find(
      (s) => s.id === selectedSkill.id,
    );
    if (updated) setSelectedSkill(updated);
  };

  const handleRemoveFromAgent = async (agentId: string) => {
    if (!selectedSkill) return;
    await removeSkillFromAgents(selectedSkill.id, [agentId]);
    await fetchItems();
    const updated = useInstalledStore.getState().items.find(
      (s) => s.id === selectedSkill.id,
    );
    if (updated) setSelectedSkill(updated);
  };

  return (
    <div className="flex flex-col h-full">
      <TopBar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        searchPlaceholder="Search installed skills..."
        onSettingsClick={() => navigate('/settings')}
      />

      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Package className="h-5 w-5 text-blue-600" />
            <span className="text-sm font-semibold text-slate-700 dark:text-slate-300">
              Installed Skills
            </span>
            {!isLoading && (
              <Badge variant="outline" className="text-xs">
                {filteredSkills.length}
              </Badge>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleBatchAssess}
              disabled={items.length === 0 || assessLoading}
            >
              <FileSearch className="h-4 w-4 mr-1" />
              {assessLoading ? 'Assessing...' : 'Batch Assess'}
            </Button>
            <select
              value={agentFilter}
              onChange={(e) => setAgentFilter(e.target.value)}
              className="h-8 text-xs rounded-md border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-300 px-2 focus:outline-none focus:ring-1 focus:ring-blue-500"
            >
              <option value="">All Agents</option>
              {agents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
              {agents.length > 0 && <option value="__none__">No Agent</option>}
            </select>
            <div className="flex border border-slate-200 dark:border-slate-700 rounded-md">
              <button
                onClick={() => setViewMode('grid')}
                className={`p-1.5 ${
                  viewMode === 'grid'
                    ? 'bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-300'
                    : 'text-slate-400 hover:text-slate-600'
                }`}
                title="Grid view"
              >
                <LayoutGrid className="h-4 w-4" />
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`p-1.5 ${
                  viewMode === 'list'
                    ? 'bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-300'
                    : 'text-slate-400 hover:text-slate-600'
                }`}
                title="List view"
              >
                <List className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>

        {/* Safety Filter Tabs */}
        <Tabs
          defaultValue="all"
          value={safetyFilter}
          onValueChange={(v) => setSafetyFilter(v as SafetyFilter)}
        >
          <TabsList>
            <TabsTrigger value="all">
              <Shield className="h-4 w-4 mr-1" />
              All
            </TabsTrigger>
            <TabsTrigger value="safe">
              <ShieldCheck className="h-4 w-4 mr-1" />
              Safe
            </TabsTrigger>
            <TabsTrigger value="warning">
              <ShieldAlert className="h-4 w-4 mr-1" />
              Warning
            </TabsTrigger>
            <TabsTrigger value="danger">
              <ShieldAlert className="h-4 w-4 mr-1" />
              Danger
            </TabsTrigger>
          </TabsList>
        </Tabs>

        {/* Batch Assessment Summary */}
        {summary && (
          <div className="rounded-lg border border-blue-200 dark:border-blue-800 bg-blue-50 dark:bg-blue-950 p-3 flex items-center justify-between">
            <div className="flex items-center gap-4 text-sm">
              <span className="text-blue-700 dark:text-blue-300">
                Assessed {summary.total} skills
              </span>
              <span className="text-green-600 dark:text-green-400">
                Safe: {summary.safe_count}
              </span>
              <span className="text-yellow-600 dark:text-yellow-400">
                Warning: {summary.warning_count}
              </span>
              <span className="text-red-600 dark:text-red-400">
                Danger: {summary.dangerous_count}
              </span>
            </div>
          </div>
        )}

        {/* Content: Grid or List */}
        {isLoading && viewMode === 'grid' && (
          <LoadingSkeleton variant="card" count={6} />
        )}
        {isLoading && viewMode === 'list' && (
          <LoadingSkeleton variant="list" count={6} />
        )}

        {error && <div>Error: {error}</div>}

        {!isLoading && items.length === 0 && (
          <EmptyState
            icon={<Package className="h-12 w-12" />}
            title="No skills installed"
            description="Discover and install skills from the marketplace to get started."
            actionLabel="Discover Skills"
            onAction={() => navigate('/discover')}
          />
        )}

        {!isLoading && items.length > 0 && filteredSkills.length === 0 && (
          <EmptyState
            icon={<Package className="h-12 w-12" />}
            title="No matching skills"
            description="Try adjusting your search or filter criteria."
          />
        )}

        {!isLoading && filteredSkills.length > 0 && viewMode === 'grid' && (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredSkills.map((skill) => (
              <SkillCard
                key={skill.id}
                skill={skill}
                enabled={skill.enabled}
                onToggle={(enabled) => handleToggle(skill.id, enabled)}
                onClick={() => {
                  setSelectedSkill(skill);
                  setDetailOpen(true);
                }}
                agentNames={skill.agentIds.map(id => agentNameMap[id]).filter(Boolean)}
              />
            ))}
          </div>
        )}

        {!isLoading && filteredSkills.length > 0 && viewMode === 'list' && (
          <div className="space-y-2">
            {filteredSkills.map((skill) => (
              <div
                key={skill.id}
                onClick={() => {
                  setSelectedSkill(skill);
                  setDetailOpen(true);
                }}
                className="flex items-center gap-4 rounded-lg border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 cursor-pointer hover:border-slate-300 dark:hover:border-slate-600 transition-colors"
              >
                <div className="flex items-center justify-center h-9 w-9 rounded-md bg-blue-100 dark:bg-blue-900 flex-shrink-0">
                  <Package className="h-4 w-4 text-blue-600 dark:text-blue-300" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="text-sm font-medium text-slate-800 dark:text-slate-200 truncate">
                    {skill.name}
                  </h3>
                  <p className="text-xs text-slate-500 truncate">
                    {skill.description}
                  </p>
                </div>
                <div className="flex items-center gap-3 flex-shrink-0">
                  {skill.formatScore != null && (
                    <span
                      className={`text-xs px-1.5 py-0.5 rounded font-medium ${
                        skill.formatScore >= 80
                          ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                          : skill.formatScore >= 50
                            ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300'
                            : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'
                      }`}
                    >
                      F: {skill.formatScore}
                    </span>
                  )}
                  <span
                    className={`text-xs px-1.5 py-0.5 rounded font-medium ${
                      skill.enabled
                        ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                        : 'bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-400'
                    }`}
                  >
                    {skill.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Detail Panel */}
      <SkillDetail
        skill={selectedSkill}
        open={detailOpen}
        onOpenChange={setDetailOpen}
        isInstalled={true}
        onUninstall={
          selectedSkill ? () => handleUninstall(selectedSkill.id) : undefined
        }
        agentsCount={selectedSkill?.agentCount ?? 0}
        agentNames={selectedSkill?.agentIds.map(id => agentNameMap[id]).filter(Boolean)}
        agents={agents}
        onApplyToAgent={handleApplyToAgent}
        onRemoveFromAgent={handleRemoveFromAgent}
      />

      {/* Conflict Resolver */}
      <ConflictResolver
        open={conflictOpen}
        onOpenChange={setConflictOpen}
        onAllResolved={() => fetchItems()}
      />
    </div>
  );
}

export { InstalledPage };

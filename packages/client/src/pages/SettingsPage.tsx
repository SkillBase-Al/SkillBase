import * as React from 'react';
import {
  Settings,
  Users,
  HardDrive,
  Network,
  Info,
  Plus,
  Trash2,
  FolderOpen,
  Globe,
  Server,
  Brain,
  RefreshCw,
} from 'lucide-react';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../components/ui/tabs';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Card, CardContent, CardHeader } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Switch } from '../components/ui/switch';
import { AgentConfigForm } from '../components/agent/AgentConfigForm';
import { LoadingSkeleton } from '../components/shared/LoadingSkeleton';
import { useAgentStore } from '../stores/agentStore';
import { useSettingsStore } from '../stores/settingsStore';
import { scanLocalSkills } from '../services/tauri';
import { useInstalledStore } from '../stores/installedStore';
import type { AgentConfig } from '../types/agent';

function SettingsPage() {
  const {
    agents,
    isLoading: agentsLoading,
    fetchAgents,
    addAgent,
    updateAgent,
    deleteAgent,
  } = useAgentStore();
  const {
    settings,
    isLoading: settingsLoading,
    fetchSettings,
    updateSettings,
  } = useSettingsStore();

  const [activeTab, setActiveTab] = React.useState('agents');
  const [showAddForm, setShowAddForm] = React.useState(false);
  const [editingAgent, setEditingAgent] = React.useState<AgentConfig | null>(null);
  const [serverUrl, setServerUrl] = React.useState('');
  const [proxyUrl, setProxyUrl] = React.useState('');
  const [autoScan, setAutoScan] = React.useState(false);
  const [autoAssess, setAutoAssess] = React.useState(false);

  const { fetchItems: fetchInstalled, items: installedSkills } = useInstalledStore();

  React.useEffect(() => {
    fetchAgents();
    fetchSettings();
    fetchInstalled();
  }, [fetchAgents, fetchSettings, fetchInstalled]);

  React.useEffect(() => {
    if (settings) {
      setServerUrl(settings.serverUrl ?? '');
      setProxyUrl(settings.proxyUrl ?? '');
      setAutoScan(settings.autoScan ?? false);
      setAutoAssess(settings.autoAssess ?? false);
    }
  }, [settings]);

  const handleAddAgent = async (data: {
    name: string;
    type: string;
    path: string;
  }) => {
    try {
      await addAgent(data.name, data.type, data.path);
      setShowAddForm(false);
      // Automatically scan all agent paths for SKILL.md files
      const paths = data.path.split(';').map(p => p.trim()).filter(Boolean);
      await scanLocalSkills(paths);
      await fetchInstalled();
    } catch {
      // Error handled by store
    }
  };

  const handleUpdateAgent = async (data: {
    name: string;
    type: string;
    path: string;
  }) => {
    if (!editingAgent) return;
    try {
      await updateAgent(editingAgent.id, data.name, data.type, data.path);
      setEditingAgent(null);
      // Re-scan skills under the new paths
      const paths = data.path.split(';').map(p => p.trim()).filter(Boolean);
      await scanLocalSkills(paths);
      await fetchInstalled();
    } catch {
      // Error handled by store
    }
  };

  const handleDeleteAgent = async (agentId: string) => {
    try {
      await deleteAgent(agentId);
      await fetchInstalled();
    } catch {
      // Error handled by store
    }
  };

  const handleScanAgentPath = async (basePath: string) => {
    try {
      const paths = basePath.split(';').map(p => p.trim()).filter(Boolean);
      await scanLocalSkills(paths);
      await fetchInstalled();
    } catch {
      // Error handled by store
    }
  };

  const handleSaveNetwork = async () => {
    await updateSettings({ serverUrl, proxyUrl });
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Header */}
        <div className="flex items-center gap-2 mb-2">
          <Settings className="h-5 w-5 text-blue-600" />
          <span className="text-sm font-semibold text-slate-700 dark:text-slate-300">
            Settings
          </span>
        </div>

        <Tabs defaultValue="agents" value={activeTab} onValueChange={setActiveTab}>
          <TabsList>
            <TabsTrigger value="agents">
              <Users className="h-4 w-4 mr-1" />
              Agents
            </TabsTrigger>
            <TabsTrigger value="storage">
              <HardDrive className="h-4 w-4 mr-1" />
              Storage
            </TabsTrigger>
            <TabsTrigger value="network">
              <Network className="h-4 w-4 mr-1" />
              Network
            </TabsTrigger>
            <TabsTrigger value="about">
              <Info className="h-4 w-4 mr-1" />
              About
            </TabsTrigger>
          </TabsList>

          {/* Agents Tab */}
          <TabsContent value="agents" className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-slate-700 dark:text-slate-300">
                Configured Agents
              </h3>
              {!showAddForm && !editingAgent && (
                <Button
                  variant="default"
                  size="sm"
                  onClick={() => setShowAddForm(true)}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  Add Agent
                </Button>
              )}
            </div>

            {agentsLoading && <LoadingSkeleton variant="list" count={3} />}

            {!agentsLoading && agents.length === 0 && !showAddForm && (
              <Card>
                <CardContent className="p-6 text-center">
                  <Brain className="h-8 w-8 text-slate-400 mx-auto mb-2" />
                  <p className="text-sm text-slate-500 dark:text-slate-400 mb-1">
                    No agents configured
                  </p>
                  <p className="text-xs text-slate-400 mb-4">
                    Add an agent to associate it with skills
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowAddForm(true)}
                  >
                    <Plus className="h-4 w-4 mr-1" />
                    Add Agent
                  </Button>
                </CardContent>
              </Card>
            )}

            {/* Agent List */}
            {!agentsLoading && agents.length > 0 && (
              <div className="space-y-2">
                {agents.map((agent) => (
                  <Card key={agent.id}>
                    <CardContent className="p-4">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <div className="flex items-center justify-center h-9 w-9 rounded-md bg-purple-100 dark:bg-purple-900">
                            <Brain className="h-4 w-4 text-purple-600 dark:text-purple-300" />
                          </div>
                          <div>
                            <div className="flex items-center gap-2">
                              <span className="text-sm font-medium text-slate-800 dark:text-slate-200">
                                {agent.name}
                              </span>
                              <Badge variant="outline" className="text-[10px]">
                                {agent.agentType}
                              </Badge>
                              {(() => {
                                const count = installedSkills.filter(s => s.agentIds.includes(agent.id)).length;
                                return count > 0 ? (
                                  <span className="text-xs text-slate-400 ml-1">
                                    {count} skills
                                  </span>
                                ) : null;
                              })()}
                            </div>
                            <div className="text-xs text-slate-500 mt-0.5 font-mono leading-relaxed">
                              {agent.basePath.split(';').map((p, i) => (
                                <div key={i}>{p.trim()}</div>
                              ))}
                            </div>
                          </div>
                        </div>
                        <div className="flex items-center gap-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => setEditingAgent(agent)}
                          >
                            Edit
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleScanAgentPath(agent.basePath)}
                            title="Scan this path for skills"
                          >
                            <RefreshCw className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDeleteAgent(agent.id)}
                            className="text-red-500 hover:text-red-700"
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}

            {/* Add Form */}
            {showAddForm && (
              <Card>
                <CardHeader>
                  <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300">
                    New Agent
                  </h4>
                </CardHeader>
                <CardContent>
                  <AgentConfigForm
                    onSubmit={handleAddAgent}
                    onCancel={() => setShowAddForm(false)}
                  />
                </CardContent>
              </Card>
            )}

            {/* Edit Form */}
            {editingAgent && (
              <Card>
                <CardHeader>
                  <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300">
                    Edit Agent
                  </h4>
                </CardHeader>
                <CardContent>
                  <AgentConfigForm
                    initialData={editingAgent}
                    onSubmit={handleUpdateAgent}
                    onCancel={() => setEditingAgent(null)}
                  />
                </CardContent>
              </Card>
            )}
          </TabsContent>

          {/* Storage Tab */}
          <TabsContent value="storage" className="space-y-4">
            <Card>
              <CardHeader>
                <h3 className="text-sm font-medium text-slate-700 dark:text-slate-300">
                  Storage Settings
                </h3>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-slate-700 dark:text-slate-300">
                      Auto-scan on startup
                    </p>
                    <p className="text-xs text-slate-500">
                      Automatically scan configured paths when app starts
                    </p>
                  </div>
                  <Switch
                    checked={autoScan}
                    onCheckedChange={(v) => {
                      setAutoScan(v);
                      updateSettings({ autoScan: v });
                    }}
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-slate-700 dark:text-slate-300">
                      Auto-assess on import
                    </p>
                    <p className="text-xs text-slate-500">
                      Automatically assess format and security when importing
                    </p>
                  </div>
                  <Switch
                    checked={autoAssess}
                    onCheckedChange={(v) => {
                      setAutoAssess(v);
                      updateSettings({ autoAssess: v });
                    }}
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Network Tab */}
          <TabsContent value="network" className="space-y-4">
            <Card>
              <CardHeader>
                <h3 className="text-sm font-medium text-slate-700 dark:text-slate-300">
                  Network Settings
                </h3>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-1.5">
                  <label className="text-sm font-medium text-slate-700 dark:text-slate-300 flex items-center gap-2">
                    <Server className="h-4 w-4 text-slate-400" />
                    Server URL
                  </label>
                  <Input
                    value={serverUrl}
                    onChange={(e) => setServerUrl(e.target.value)}
                    placeholder="https://api.skillbase.example.com"
                  />
                  <p className="text-xs text-slate-400">
                    The URL of the SkillBase marketplace server
                  </p>
                </div>
                <div className="space-y-1.5">
                  <label className="text-sm font-medium text-slate-700 dark:text-slate-300 flex items-center gap-2">
                    <Globe className="h-4 w-4 text-slate-400" />
                    Proxy URL
                  </label>
                  <Input
                    value={proxyUrl}
                    onChange={(e) => setProxyUrl(e.target.value)}
                    placeholder="http://proxy.example.com:8080"
                  />
                  <p className="text-xs text-slate-400">
                    Optional proxy for outbound connections
                  </p>
                </div>
                <Button variant="default" size="sm" onClick={handleSaveNetwork}>
                  Save Network Settings
                </Button>
              </CardContent>
            </Card>
          </TabsContent>

          {/* About Tab */}
          <TabsContent value="about">
            <Card>
              <CardContent className="p-6 space-y-4">
                <div className="flex items-center gap-3">
                  <div className="flex items-center justify-center h-12 w-12 rounded-xl bg-blue-600 text-white">
                    <Brain className="h-6 w-6" />
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
                      SkillBase
                    </h3>
                    <p className="text-sm text-slate-500">
                      AI Skill Management System
                    </p>
                  </div>
                </div>
                <div className="border-t border-slate-200 dark:border-slate-700 pt-4 space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-500">Version</span>
                    <span className="text-slate-700 dark:text-slate-300 font-mono">
                      0.1.0
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-500">Framework</span>
                    <span className="text-slate-700 dark:text-slate-300">
                      Tauri 2.0
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-500">Frontend</span>
                    <span className="text-slate-700 dark:text-slate-300">
                      React 18 + TypeScript
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-500">UI</span>
                    <span className="text-slate-700 dark:text-slate-300">
                      Tailwind CSS + shadcn/ui
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

export { SettingsPage };

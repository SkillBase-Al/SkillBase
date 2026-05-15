import * as React from 'react';
import {
  Brain,
  FolderSearch,
  Download,
  ChevronLeft,
  ChevronRight,
  Check,
  FolderOpen,
} from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Card, CardContent } from '../ui/card';
import { cn } from '../../lib/utils';
import { useToast } from '../ui/toast';
import { scanLocalSkills, importSkills, addAgent } from '../../services/tauri';
import type { AgentConfig } from '../../types/agent';
import type { InstalledSkill } from '../../types/skill';

interface FirstRunWizardProps {
  onComplete: () => void;
}

type Step = 1 | 2 | 3;

const agentPresets: Array<{
  type: string;
  label: string;
  defaultPaths: string[];
}> = [
  {
    type: 'cursor',
    label: 'Cursor',
    defaultPaths: ['~/.cursor/skills/'],
  },
  {
    type: 'claude',
    label: 'Claude Code',
    defaultPaths: ['~/.claude/skills/'],
  },
  {
    type: 'windsurf',
    label: 'Windsurf',
    defaultPaths: ['~/.codeium/windsurf/global_workflows/'],
  },
  {
    type: 'qoder',
    label: 'Qoder',
    defaultPaths: ['~/.qoder/skills', '~/.agents/skills'],
  },
  {
    type: 'opencode',
    label: 'OpenCode',
    defaultPaths: ['~/.config/opencode/skills/'],
  },
];

function FirstRunWizard({ onComplete }: FirstRunWizardProps) {
  const [currentStep, setCurrentStep] = React.useState<Step>(1);
  const [selectedAgents, setSelectedAgents] = React.useState<string[]>([]);
  const [folders, setFolders] = React.useState<string[]>(['']);
  const [scanResults, setScanResults] = React.useState<InstalledSkill[]>([]);
  const [selectedSkills, setSelectedSkills] = React.useState<string[]>([]);
  const [isScanning, setIsScanning] = React.useState(false);
  const [isImporting, setIsImporting] = React.useState(false);
  const [scanError, setScanError] = React.useState<string | null>(null);
  const { addToast } = useToast();

  const handleToggleAgent = (type: string) => {
    setSelectedAgents((prev) =>
      prev.includes(type) ? prev.filter((t) => t !== type) : [...prev, type],
    );
  };

  const handleAddFolder = () => {
    setFolders((prev) => [...prev, '']);
  };

  const handleFolderChange = (index: number, value: string) => {
    setFolders((prev) => {
      const next = [...prev];
      next[index] = value;
      return next;
    });
  };

  const handleRemoveFolder = (index: number) => {
    setFolders((prev) => prev.filter((_, i) => i !== index));
  };

  const handleScan = async () => {
    const validPaths = folders.filter((f) => f.trim().length > 0);
    if (validPaths.length === 0) {
      addToast('Please add at least one folder to scan', 'warning');
      return;
    }

    setIsScanning(true);
    setScanError(null);
    try {
      const results = await scanLocalSkills(validPaths);
      setScanResults(results);
      setSelectedSkills(results.map((s) => s.id));
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to scan folders';
      setScanError(msg);
      addToast(msg, 'error');
    } finally {
      setIsScanning(false);
    }
  };

  const createSelectedAgents = async () => {
    for (const type of selectedAgents) {
      const preset = agentPresets.find((p) => p.type === type);
      if (preset) {
        try {
          const path = preset.defaultPaths.join(';');
          await addAgent(preset.label, preset.type, path);
        } catch {
          // Ignore individual agent creation errors
        }
      }
    }
  };

  const finish = async () => {
    await createSelectedAgents();
    onComplete();
  };

  const handleImport = async () => {
    if (selectedSkills.length === 0) return;

    setIsImporting(true);
    try {
      await importSkills(selectedSkills);
      addToast(
        `Successfully imported ${selectedSkills.length} skill(s)`,
        'success',
      );
      await finish();
    } catch (err) {
      const msg =
        err instanceof Error ? err.message : 'Failed to import skills';
      addToast(msg, 'error');
    } finally {
      setIsImporting(false);
    }
  };

  const canProceed = () => {
    if (currentStep === 1) return true;
    if (currentStep === 2) {
      return folders.some((f) => f.trim().length > 0);
    }
    return true;
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <Card className="w-full max-w-lg mx-4 max-h-[85vh] overflow-y-auto">
        {/* Step indicator */}
        <div className="flex items-center justify-center gap-2 pt-6 pb-2">
          {([1, 2, 3] as Step[]).map((step) => (
            <React.Fragment key={step}>
              <div
                className={cn(
                  'flex items-center justify-center h-8 w-8 rounded-full text-sm font-medium transition-colors',
                  currentStep === step
                    ? 'bg-blue-600 text-white'
                    : currentStep > step
                      ? 'bg-green-500 text-white'
                      : 'bg-slate-200 dark:bg-slate-700 text-slate-500',
                )}
              >
                {currentStep > step ? (
                  <Check className="h-4 w-4" />
                ) : (
                  step
                )}
              </div>
              {step < 3 && (
                <div
                  className={cn(
                    'h-0.5 w-12 rounded',
                    currentStep > step
                      ? 'bg-green-500'
                      : 'bg-slate-200 dark:bg-slate-700',
                  )}
                />
              )}
            </React.Fragment>
          ))}
        </div>

        {/* Step 1: Configure Agent Paths */}
        {currentStep === 1 && (
          <CardContent className="space-y-4">
            <div className="text-center mb-4">
              <div className="flex justify-center mb-3">
                <div className="p-3 rounded-full bg-blue-100 dark:bg-blue-900">
                  <Brain className="h-6 w-6 text-blue-600 dark:text-blue-300" />
                </div>
              </div>
              <h2 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
                Select AI Agents
              </h2>
              <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                Choose which AI agent types you use to get started
              </p>
            </div>

            {agentPresets.map((preset) => (
              <button
                key={preset.type}
                onClick={() => handleToggleAgent(preset.type)}
                className={cn(
                  'w-full text-left p-3 rounded-lg border transition-colors',
                  selectedAgents.includes(preset.type)
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-950'
                    : 'border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800',
                )}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <span className="font-medium text-sm text-slate-800 dark:text-slate-200">
                      {preset.label}
                    </span>
                    <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                      Default: {preset.defaultPaths.join(', ')}
                    </p>
                  </div>
                  <div
                    className={cn(
                      'h-5 w-5 rounded border-2 flex items-center justify-center transition-colors',
                      selectedAgents.includes(preset.type)
                        ? 'border-blue-600 bg-blue-600'
                        : 'border-slate-300 dark:border-slate-600',
                    )}
                  >
                    {selectedAgents.includes(preset.type) && (
                      <Check className="h-3 w-3 text-white" />
                    )}
                  </div>
                </div>
              </button>
            ))}
          </CardContent>
        )}

        {/* Step 2: Scan Folders */}
        {currentStep === 2 && (
          <CardContent className="space-y-4">
            <div className="text-center mb-4">
              <div className="flex justify-center mb-3">
                <div className="p-3 rounded-full bg-purple-100 dark:bg-purple-900">
                  <FolderSearch className="h-6 w-6 text-purple-600 dark:text-purple-300" />
                </div>
              </div>
              <h2 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
                Scan for Skills
              </h2>
              <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                Choose folders to scan for SKILL.md files
              </p>
            </div>

            <div className="space-y-2">
              {folders.map((folder, index) => (
                <div key={index} className="flex items-center gap-2">
                  <Input
                    value={folder}
                    onChange={(e) => handleFolderChange(index, e.target.value)}
                    placeholder={`Folder path ${index + 1}`}
                    className="flex-1"
                  />
                  <button
                    onClick={() => handleRemoveFolder(index)}
                    className="text-slate-400 hover:text-red-500 p-1"
                    disabled={folders.length === 1}
                  >
                    <svg
                      className="h-4 w-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M6 18L18 6M6 6l12 12"
                      />
                    </svg>
                  </button>
                </div>
              ))}
            </div>

            <button
              onClick={handleAddFolder}
              className="flex items-center gap-2 text-sm text-blue-600 dark:text-blue-400 hover:text-blue-700"
            >
              <FolderOpen className="h-4 w-4" />
              Add another folder
            </button>

            {scanError && (
              <p className="text-sm text-red-500">{scanError}</p>
            )}

            {scanResults.length > 0 && (
              <div className="rounded-lg border border-slate-200 dark:border-slate-700 p-3">
                <p className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
                  Found {scanResults.length} skill(s)
                </p>
                {scanResults.slice(0, 5).map((skill) => (
                  <div
                    key={skill.id}
                    className="text-xs text-slate-500 dark:text-slate-400 py-0.5"
                  >
                    {skill.name} (v{skill.version ?? 'N/A'})
                  </div>
                ))}
                {scanResults.length > 5 && (
                  <p className="text-xs text-slate-400 mt-1">
                    ...and {scanResults.length - 5} more
                  </p>
                )}
              </div>
            )}
          </CardContent>
        )}

        {/* Step 3: Import Skills */}
        {currentStep === 3 && (
          <CardContent className="space-y-4">
            <div className="text-center mb-4">
              <div className="flex justify-center mb-3">
                <div className="p-3 rounded-full bg-green-100 dark:bg-green-900">
                  <Download className="h-6 w-6 text-green-600 dark:text-green-300" />
                </div>
              </div>
              <h2 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
                Import Skills
              </h2>
              <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                Select which discovered skills to import
              </p>
            </div>

            {scanResults.length === 0 ? (
              <div className="text-center py-8 text-slate-400">
                <p className="text-sm">
                  No skills found. Go back and scan folders first.
                </p>
              </div>
            ) : (
              <div className="space-y-2 max-h-60 overflow-y-auto">
                {scanResults.map((skill) => (
                  <label
                    key={skill.id}
                    className="flex items-center gap-3 p-2 rounded-lg border border-slate-200 dark:border-slate-700 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800"
                  >
                    <input
                      type="checkbox"
                      checked={selectedSkills.includes(skill.id)}
                      onChange={() =>
                        setSelectedSkills((prev) =>
                          prev.includes(skill.id)
                            ? prev.filter((s) => s !== skill.id)
                            : [...prev, skill.id],
                        )
                      }
                      className="h-4 w-4 rounded border-slate-300 text-blue-600"
                    />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-slate-800 dark:text-slate-200 truncate">
                        {skill.name}
                      </p>
                      <p className="text-xs text-slate-500 truncate">
                        {skill.description}
                      </p>
                    </div>
                    <span className="text-xs text-slate-400">
                      {skill.version ? `v${skill.version}` : ''}
                    </span>
                  </label>
                ))}
              </div>
            )}
          </CardContent>
        )}

        {/* Footer */}
        <div className="flex items-center justify-between p-6 pt-2 border-t border-slate-200 dark:border-slate-700">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                if (currentStep > 1) {
                  setCurrentStep((currentStep - 1) as Step);
                }
              }}
              disabled={currentStep === 1}
            >
              <ChevronLeft className="h-4 w-4 mr-1" />
              Back
            </Button>
            {currentStep >= 2 && (
              <Button
                variant="outline"
                size="sm"
                onClick={finish}
              >
                Skip & Finish
              </Button>
            )}
          </div>

          {currentStep < 3 ? (
            <Button
              size="sm"
              disabled={!canProceed() || isScanning}
              onClick={async () => {
                if (currentStep === 1) {
                  // Pre-fill folder paths from selected agents
                  const agentPaths = selectedAgents.flatMap((type) => {
                    const preset = agentPresets.find((p) => p.type === type);
                    return preset ? preset.defaultPaths : [];
                  });
                  setFolders(agentPaths.length > 0 ? agentPaths : ['']);
                  setScanResults([]);
                  setSelectedSkills([]);
                }
                if (currentStep === 2 && scanResults.length === 0) {
                  await handleScan();
                }
                if (!isScanning) {
                  setCurrentStep((currentStep + 1) as Step);
                }
              }}
            >
              {currentStep === 2 && scanResults.length === 0
                ? isScanning
                  ? 'Scanning...'
                  : 'Scan'
                : 'Continue'}
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          ) : (
            <Button
              size="sm"
              disabled={selectedSkills.length === 0 || isImporting}
              onClick={handleImport}
            >
              {isImporting ? (
                'Importing...'
              ) : (
                <>
                  Import {selectedSkills.length} skill(s)
                  <ChevronRight className="h-4 w-4 ml-1" />
                </>
              )}
            </Button>
          )}
        </div>
      </Card>
    </div>
  );
}

export { FirstRunWizard };

import { invoke } from '@tauri-apps/api/core';
import type { InstalledSkill, MarketSkill } from '../types/skill';
import type { AgentConfig } from '../types/agent';
import type {
  FormatResult,
  SecurityResult,
  AssessmentSummary,
} from '../types/assessment';
import type { DedupGroup } from '../types/dedup';
import type { SkillConflict } from '../types/conflict';
import type { AppSettings } from '../types/settings';

// ─── Installed Skills ───────────────────────────────────────────

export async function getInstalledSkills(): Promise<InstalledSkill[]> {
  return invoke<InstalledSkill[]>('get_installed_skills');
}

export async function installSkill(skillId: string): Promise<InstalledSkill> {
  // First fetch full skill detail from market
  const detail = await getSkillDetail(skillId);
  // Then invoke install with all required fields (camelCase for Tauri v2)
  return invoke<InstalledSkill>('install_skill', {
    name: detail.name,
    description: detail.description,
    source: detail.version || 'unknown',
    sourceUrl: null,
    version: detail.version || null,
    license: null,
    author: detail.author || null,
    skillContent: '',
    agentIds: [],
  });
}

export async function uninstallSkill(skillId: string): Promise<void> {
  return invoke<void>('uninstall_skill', { id: skillId });
}

export async function updateSkill(skillId: string, skillContent: string): Promise<void> {
  return invoke<void>('update_skill', { id: skillId, skillContent });
}

export async function applySkillToAgents(
  skillId: string,
  agentIds: string[],
): Promise<void> {
  return invoke<void>('apply_skill_to_agents', { skillId, agentIds });
}

export async function removeSkillFromAgents(
  skillId: string,
  agentIds: string[],
): Promise<void> {
  return invoke<void>('remove_skill_from_agents', { skillId, agentIds });
}

export async function toggleSkillEnabled(
  skillId: string,
  enabled: boolean,
): Promise<void> {
  return invoke<void>('toggle_skill_enabled', { id: skillId, enabled });
}

export async function scanLocalSkills(paths: string[]): Promise<InstalledSkill[]> {
  return invoke<InstalledSkill[]>('scan_local_skills', { paths });
}

export async function importSkills(paths: string[]): Promise<InstalledSkill[]> {
  return invoke<InstalledSkill[]>('import_skills', { paths });
}

// ─── Assessment ──────────────────────────────────────────────────

export async function assessFormat(skillId: string): Promise<FormatResult> {
  return invoke<FormatResult>('assess_format', { skillId });
}

export async function assessSecurity(skillId: string): Promise<SecurityResult> {
  return invoke<SecurityResult>('assess_security', { skillId });
}

export async function batchAssess(): Promise<AssessmentSummary> {
  return invoke<AssessmentSummary>('batch_assess');
}

export async function getAssessmentResults(
  skillId?: string,
): Promise<(FormatResult | SecurityResult)[]> {
  return invoke('get_assessment_results', { skillId });
}

// ─── Dedup ───────────────────────────────────────────────────────

export async function runDedup(): Promise<DedupGroup[]> {
  return invoke<DedupGroup[]>('run_dedup');
}

export async function getDedupGroups(): Promise<DedupGroup[]> {
  return invoke<DedupGroup[]>('get_dedup_groups');
}

export async function deleteSkillFromGroup(
  _groupId: string,
  skillId: string,
): Promise<void> {
  return invoke<void>('delete_skill_from_group', { skillId });
}

// ─── Agents ──────────────────────────────────────────────────────

export async function getAgents(): Promise<AgentConfig[]> {
  return invoke<AgentConfig[]>('get_agents');
}

export async function addAgent(name: string, agentType: string, basePath: string): Promise<AgentConfig> {
  return invoke<AgentConfig>('add_agent', { name, agentType: agentType, basePath: basePath });
}

export async function updateAgent(id: string, name: string, agentType: string, basePath: string): Promise<void> {
  return invoke<void>('update_agent', { id, name, agentType, basePath });
}

export async function deleteAgent(agentId: string): Promise<void> {
  return invoke<void>('delete_agent', { id: agentId });
}

// ─── Market ──────────────────────────────────────────────────────

// Raw types matching Rust server structs (snake_case)
interface RawMarketSkill {
  id: string;
  name: string;
  description: string | null;
  source: string;
  source_url: string | null;
  license: string | null;
  author: string | null;
  rating: number | null;
  install_count: number | null;
  categories: string[];
  safety_level: string | null;
  format_score: number | null;
  quality_score: number | null;
  skill_md_content: string | null;
  created_at: string | null;
  updated_at: string | null;
}

interface RawPaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
}

interface RawCategory {
  id: string;
  name: string;
  display_name: string;
}

/** Parse the skill name from SKILL.md YAML frontmatter, falling back to the path. */
function parseSkillName(rawName: string, content?: string | null): string {
  if (content) {
    const match = content.match(/^---\s*\nname:\s*(.+)/);
    if (match) return match[1].trim();
  }
  // Fallback: extract the directory name before "/SKILL.md"
  const idx = rawName.lastIndexOf('/');
  if (idx !== -1) {
    const parent = rawName.lastIndexOf('/', idx - 1);
    return rawName.slice(parent !== -1 ? parent + 1 : 0, idx);
  }
  return rawName;
}

/** Parse the skill description from SKILL.md YAML frontmatter. */
function parseDescription(rawDescription: string | null, content?: string | null): string {
  if (rawDescription) return rawDescription;
  if (content) {
    const match = content.match(/^description:\s*"(.+)"\s*$/m);
    if (match) return match[1];
    const simple = content.match(/^description:\s*(.+)$/m);
    if (simple) return simple[1];
  }
  return '';
}

function toMarketSkill(raw: RawMarketSkill): MarketSkill {
  const skillContent = raw.skill_md_content ?? undefined;
  return {
    id: raw.id,
    name: parseSkillName(raw.name, skillContent),
    description: parseDescription(raw.description, skillContent),
    version: '',
    source: raw.source,
    packageId: raw.id,
    author: raw.author ?? '',
    skillContent,
    tags: raw.source ? [raw.source] : [],
    category: raw.categories?.[0] ?? '',
    downloads: raw.install_count ?? 0,
    rating: raw.rating ?? 0,
    isInstalled: false,
    createdAt: raw.created_at ?? '',
    updatedAt: raw.updated_at ?? '',
    formatScore: raw.format_score ?? undefined,
    safetyLevel: raw.safety_level ?? undefined,
  };
}

export async function searchMarket(
  query: string,
  category?: string,
  page: number = 1,
  perPage: number = 15,
): Promise<{ skills: MarketSkill[]; total: number }> {
  const resp = await invoke<RawPaginatedResponse<RawMarketSkill>>('search_market', { query, category, page, perPage });
  return { skills: resp.data.map(toMarketSkill), total: resp.total };
}

export async function getSkillDetail(
  skillId: string,
): Promise<MarketSkill> {
  const raw = await invoke<RawMarketSkill>('get_skill_detail', { skillId });
  return toMarketSkill(raw);
}

export async function getCategories(): Promise<string[]> {
  const cats = await invoke<RawCategory[]>('get_categories');
  return cats.map((c) => c.name);
}

// ─── Skill Content ────────────────────────────────────────────────

export async function getSkillFileContent(skillId: string): Promise<string> {
  return invoke<string>('get_skill_content', { skillId });
}

// ─── Conflict Resolution ──────────────────────────────────────────

export async function getSkillConflicts(): Promise<SkillConflict[]> {
  return invoke<SkillConflict[]>('get_skill_conflicts');
}

export async function resolveSkillConflict(
  conflictId: string,
  keptCandidateId: string,
): Promise<void> {
  return invoke<void>('resolve_skill_conflict', { conflictId, keptCandidateId });
}

// ─── Settings ────────────────────────────────────────────────────

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings');
}

export async function updateSettings(
  settings: AppSettings,
): Promise<AppSettings> {
  return invoke<AppSettings>('update_settings', { settings });
}

export async function checkFirstRun(): Promise<boolean> {
  return invoke<boolean>('check_first_run');
}

// ─── Feedback ──────────────────────────────────────────────────

export async function submitFeedback(title: string, description: string): Promise<void> {
  return invoke<void>('submit_feedback', { title, description });
}

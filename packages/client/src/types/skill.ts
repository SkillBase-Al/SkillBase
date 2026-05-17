export interface Skill {
  id: string;
  name: string;
  description: string;
  version?: string;
  author?: string;
  packageId: string;
  path: string;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  skillContent?: string;
}

export interface InstalledSkill {
  id: string;
  name: string;
  description: string;
  version?: string;
  author?: string;
  path: string;
  enabled: boolean;
  tags: string[];
  formatScore?: number;
  safetyLevel?: string;
  qualityScore?: number;
  hasSecurityIssues: boolean;
  installedAt: string;
  updatedAt?: string;
  agentCount: number;
  agentIds: string[];
  skillContent?: string;
}

export interface MarketSkill {
  id: string;
  name: string;
  description: string;
  version: string;
  packageId: string;
  author: string;
  source: string;
  tags: string[];
  category: string;
  downloads: number;
  rating: number;
  isInstalled: boolean;
  formatScore?: number;
  safetyLevel?: string;
  createdAt: string;
  updatedAt: string;
  skillContent?: string;
}

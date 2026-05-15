export interface ConflictCandidate {
  id: string;
  skillId: string | null;
  skillDir: string;
  agentId: string;
  agentName: string;
  agentType: string;
  version: string | null;
  contentHash: string;
}

export interface SkillConflict {
  id: string;
  skillName: string;
  candidates: ConflictCandidate[];
  resolved: boolean;
  keptCandidateId: string | null;
  createdAt: string;
}

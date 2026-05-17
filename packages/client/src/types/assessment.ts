export interface Issue {
  line: number;
  column: number;
  severity: 'error' | 'warning' | 'info';
  message: string;
  code?: string;
}

export interface FormatResult {
  skillId: string;
  skillName: string;
  score: number;
  issues: Issue[];
  hasFrontmatter: boolean;
  hasDescription: boolean;
  hasExamples: boolean;
  hasValidStructure: boolean;
  assessedAt: string;
}

export interface SecurityResult {
  skillId: string;
  skillName: string;
  score: number;
  issues: Issue[];
  hasExecPermissions: boolean;
  hasNetworkAccess: boolean;
  hasFileSystemAccess: boolean;
  hasDangerousPatterns: boolean;
  assessedAt: string;
}

export interface DepResult {
  skillId: string;
  skillName: string;
  score: number;
  issues: Issue[];
  outdatedCount: number;
  deprecatedCount: number;
  assessedAt: string;
}

export interface AssessmentResultItem {
  skillId: string;
  skillName: string;
  passed: boolean;
  formatScore: number;
  securityScore: number;
}

export interface AssessmentSummary {
  total: number;
  passed: number;
  average_format_score: number;
  average_quality_score: number;
  safe_count: number;
  warning_count: number;
  dangerous_count: number;
  // Extended fields used by the dashboard
  totalAssessed: number;
  passedFormat: number;
  passedSecurity: number;
  totalIssues: number;
  averageFormatScore: number;
  averageSecurityScore: number;
  results: AssessmentResultItem[];
}

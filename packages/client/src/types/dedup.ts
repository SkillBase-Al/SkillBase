export interface DedupGroup {
  id: string;
  similarity: number;
  skills: Array<{
    id: string;
    name: string;
    description: string;
    path: string;
    version?: string;
  }>;
  detectedAt: string;
}

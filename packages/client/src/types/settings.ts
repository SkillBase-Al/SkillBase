export interface AppSettings {
  theme: 'light' | 'dark' | 'system';
  serverUrl: string;
  proxyUrl: string;
  autoScan: boolean;
  autoAssess: boolean;
  scanPaths: string[];
  sidebarCollapsed: boolean;
  lastUpdated: string | null;
  firstRunComplete: boolean;
  crawlRepos: string[];
}

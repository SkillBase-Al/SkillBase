const API_BASE = import.meta.env.DEV ? 'http://localhost:3007' : '';

function getToken(): string | null {
  return localStorage.getItem('admin_token');
}

export function setToken(token: string) {
  localStorage.setItem('admin_token', token);
}

export function clearToken() {
  localStorage.removeItem('admin_token');
}

async function request<T>(path: string, opts?: RequestInit): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(`${API_BASE}${path}`, {
    headers: { ...headers, ...(opts?.headers as Record<string, string> | undefined) },
    ...opts,
  });

  if (res.status === 401) {
    clearToken();
    window.location.reload();
    throw new Error('Authentication required');
  }

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || res.statusText);
  }
  return res.json();
}

export async function login(username: string, password: string): Promise<string> {
  const data = await request<{ token: string }>('/api/v1/admin/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
  setToken(data.token);
  return data.token;
}

export interface DauCount {
  date: string;
  count: number;
}

export interface PvCount {
  date: string;
  count: number;
}

export interface PageRank {
  page: string;
  count: number;
}

export interface Feedback {
  id: string;
  title: string;
  description: string;
  submitterIp: string | null;
  createdAt: string;
}

export interface Overview {
  dau: number;
  pv: number;
  totalFeedback: number;
}

export function getOverview(from?: string, to?: string) {
  const params = new URLSearchParams();
  if (from) params.set('from', from);
  if (to) params.set('to', to);
  return request<Overview>(`/api/v1/admin/stats/overview?${params}`);
}

export function getDau(from?: string, to?: string) {
  const params = new URLSearchParams();
  if (from) params.set('from', from);
  if (to) params.set('to', to);
  return request<DauCount[]>(`/api/v1/admin/stats/dau?${params}`);
}

export function getPageviews(from?: string, to?: string) {
  const params = new URLSearchParams();
  if (from) params.set('from', from);
  if (to) params.set('to', to);
  return request<PvCount[]>(`/api/v1/admin/stats/pageviews?${params}`);
}

export function getPageRanking(from?: string, to?: string) {
  const params = new URLSearchParams();
  if (from) params.set('from', from);
  if (to) params.set('to', to);
  return request<PageRank[]>(`/api/v1/admin/stats/pages?${params}`);
}

export function getFeedback() {
  return request<Feedback[]>('/api/v1/admin/feedback');
}

// --- Skills ---

export interface AdminSkill {
  id: string;
  name: string;
  description: string | null;
  source: string;
  source_url: string | null;
  license: string | null;
  safety_level: string | null;
  format_score: number | null;
  quality_score: number | null;
  rating: number | null;
  install_count: number;
  created_at: string;
  updated_at: string;
}

export interface PaginatedSkills {
  data: AdminSkill[];
  total: number;
  page: number;
  per_page: number;
}

export function getSkills(page = 1, perPage = 20) {
  const params = new URLSearchParams({ page: String(page), per_page: String(perPage) });
  return request<PaginatedSkills>(`/api/v1/admin/skills?${params}`);
}

export function sendHeartbeat() {
  return request<{ status: string }>('/api/v1/telemetry/heartbeat', { method: 'POST' });
}

export function sendPageview(page: string) {
  return request<{ status: string }>('/api/v1/telemetry/pageview', {
    method: 'POST',
    body: JSON.stringify({ page }),
  });
}

import { invoke } from '@tauri-apps/api/core';

let cachedUrl: string | null = null;

async function getServerUrl(): Promise<string> {
  if (cachedUrl) return cachedUrl;
  try {
    cachedUrl = await invoke<string>('get_server_url');
  } catch {
    cachedUrl = 'http://localhost:3007';
  }
  return cachedUrl;
}

async function post(path: string, body?: unknown) {
  try {
    const url = await getServerUrl();
    await fetch(`${url}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined,
    });
  } catch {
    // Silently ignore — server might not be running
  }
}

export function sendHeartbeat() {
  post('/api/v1/telemetry/heartbeat');
}

export function sendPageview(page: string) {
  post('/api/v1/telemetry/pageview', { page });
}

export function sendFeedbackToServer(title: string, description: string) {
  post('/api/v1/feedback', { title, description });
}

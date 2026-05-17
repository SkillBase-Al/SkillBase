const SERVER_URL = 'http://localhost:3007';

async function post(path: string, body?: unknown) {
  try {
    await fetch(`${SERVER_URL}${path}`, {
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

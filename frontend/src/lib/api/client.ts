import type {
  InputDescriptor,
  OutputDescriptor,
  Session,
  Settings,
  StartSessionRequest
} from './types';
import type { Event as BackendEvent } from './events';

const BASE = '/api';

async function http<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'content-type': 'application/json' },
    ...init
  });
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(`${res.status} ${res.statusText}: ${body}`);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const api = {
  health: () => http<{ status: string; version: string }>('/health'),
  inputs: () => http<InputDescriptor[]>('/inputs'),
  outputs: () => http<OutputDescriptor[]>('/outputs'),
  session: () => http<Session | null>('/session'),
  startSession: (req: StartSessionRequest) =>
    http<Session>('/session', { method: 'POST', body: JSON.stringify(req) }),
  stopSession: () => http<void>('/session', { method: 'DELETE' }),
  settings: () => http<Settings>('/settings'),
  patchSettings: (patch: Partial<Settings>) =>
    http<Settings>('/settings', { method: 'PATCH', body: JSON.stringify(patch) })
};

/** Subscribe to backend events over SSE. Returns an unsubscribe handle. */
export function subscribeEvents(
  handler: (e: BackendEvent) => void,
  onError?: (err: unknown) => void
): () => void {
  const source = new EventSource(`${BASE}/events`);
  source.onmessage = (msg) => {
    try {
      handler(JSON.parse(msg.data) as BackendEvent);
    } catch (err) {
      onError?.(err);
    }
  };
  source.onerror = (err) => onError?.(err);
  return () => source.close();
}

import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { api, subscribeEvents } from '$lib/api/client';
import type { InputDescriptor, OutputDescriptor, Session } from '$lib/api/types';

export const inputs = writable<InputDescriptor[]>([]);
export const outputs = writable<OutputDescriptor[]>([]);
export const activeSession = writable<Session | null>(null);
export const backendOnline = writable<boolean>(false);
export const levels = writable<{ l: number; r: number }>({ l: -120, r: -120 });

let started = false;

export async function bootstrap() {
  if (!browser || started) return;
  started = true;

  try {
    const [ins, outs, sess] = await Promise.all([api.inputs(), api.outputs(), api.session()]);
    inputs.set(ins);
    outputs.set(outs);
    activeSession.set(sess);
    backendOnline.set(true);
  } catch (err) {
    backendOnline.set(false);
    console.warn('[analog-cloud] backend offline:', err);
  }

  subscribeEvents(
    (e) => {
      switch (e.type) {
        case 'input_updated':
          inputs.update((arr) => upsert(arr, e.input, (x) => x.id));
          break;
        case 'input_removed':
          inputs.update((arr) => arr.filter((x) => x.id !== e.input_id));
          break;
        case 'output_updated':
          outputs.update((arr) => upsert(arr, e.output, (x) => x.id));
          break;
        case 'output_removed':
          outputs.update((arr) => arr.filter((x) => x.id !== e.output_id));
          break;
        case 'session_started':
        case 'session_armed':
          activeSession.set(e.session);
          break;
        case 'session_stopped':
          activeSession.set(null);
          break;
        case 'session_stats':
          activeSession.update((s) =>
            s && s.session_id === e.session_id ? { ...s, stats: e.stats } : s
          );
          break;
        case 'levels':
          levels.set({
            l: e.level.left.peak_dbfs,
            r: e.level.right.peak_dbfs
          });
          break;
      }
    },
    () => backendOnline.set(false)
  );
}

function upsert<T>(arr: T[], item: T, key: (x: T) => string): T[] {
  const id = key(item);
  const idx = arr.findIndex((x) => key(x) === id);
  if (idx === -1) return [...arr, item];
  const copy = arr.slice();
  copy[idx] = item;
  return copy;
}

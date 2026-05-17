import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { themes, themeList } from './themes';
import type { Theme, ThemeId } from './types';

const STORAGE_KEY = 'analog-cloud:theme';

function initial(): ThemeId {
  if (!browser) return 'analog-core';
  const stored = localStorage.getItem(STORAGE_KEY) as ThemeId | null;
  return stored && stored in themes ? stored : 'analog-core';
}

export const themeId = writable<ThemeId>(initial());

export const theme = derived(themeId, ($id) => themes[$id] ?? themes['analog-core']);

themeId.subscribe(($id) => {
  if (!browser) return;
  localStorage.setItem(STORAGE_KEY, $id);
  applyTheme(themes[$id]);
});

export function setTheme(id: ThemeId) {
  themeId.set(id);
}

export function cycleTheme() {
  const current = get(themeId);
  const idx = themeList.findIndex((t) => t.id === current);
  const next = themeList[(idx + 1) % themeList.length];
  themeId.set(next.id);
}

export function applyTheme(t: Theme) {
  if (!browser) return;
  const root = document.documentElement;
  root.dataset.theme = t.id;
  const c = t.color;
  const s = t.surface;
  const m = t.motion;
  const ty = t.typography;
  const v = t.visualizer;
  const vars: Record<string, string> = {
    '--ac-bg': c.bg,
    '--ac-bg-elevated': c.bgElevated,
    '--ac-panel': c.panel,
    '--ac-border': c.border,
    '--ac-text': c.text,
    '--ac-text-muted': c.textMuted,
    '--ac-accent': c.accent,
    '--ac-accent-muted': c.accentMuted,
    '--ac-indicator': c.indicator,
    '--ac-danger': c.danger,
    '--ac-radius': s.radius,
    '--ac-radius-lg': s.radiusLarge,
    '--ac-chrome': s.chrome,
    '--ac-shadow': s.shadow,
    '--ac-backdrop': s.backdrop,
    '--ac-motion-fast': m.fast,
    '--ac-motion-med': m.medium,
    '--ac-motion-slow': m.slow,
    '--ac-easing': m.easing,
    '--ac-font': ty.family,
    '--ac-font-display': ty.familyDisplay,
    '--ac-font-mono': ty.familyMono,
    '--ac-weight-body': String(ty.weightBody),
    '--ac-weight-display': String(ty.weightDisplay),
    '--ac-tracking': ty.tracking,
    '--ac-viz-primary': v.primary,
    '--ac-viz-secondary': v.secondary
  };
  for (const [k, val] of Object.entries(vars)) {
    root.style.setProperty(k, val);
  }
}

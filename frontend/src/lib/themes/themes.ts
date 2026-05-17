import type { Theme, ThemeId } from './types';

const baseMotion = {
  fast: '120ms',
  medium: '220ms',
  slow: '420ms',
  easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)'
};

export const analogCore: Theme = {
  id: 'analog-core',
  name: 'Analog Core',
  blurb: 'Matte charcoal, brushed dark metal, amber accents. Flagship.',
  color: {
    bg: '#0B0B0D',
    bgElevated: '#141417',
    panel: '#1A1A1E',
    border: '#2A2A30',
    text: '#EDEDF0',
    textMuted: '#8C8C95',
    accent: '#F0A23B',
    accentMuted: '#A66F25',
    indicator: '#5BE49B',
    danger: '#FF5C5C'
  },
  typography: {
    family: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    familyDisplay: '"Inter Display", Inter, sans-serif',
    familyMono: '"JetBrains Mono", "SF Mono", Menlo, monospace',
    weightBody: 300,
    weightDisplay: 200,
    tracking: '0.02em'
  },
  surface: {
    radius: '10px',
    radiusLarge: '18px',
    chrome: 'linear-gradient(180deg, #1F1F23 0%, #141418 100%)',
    shadow: '0 1px 0 rgba(255,255,255,0.04) inset, 0 8px 24px rgba(0,0,0,0.5)',
    backdrop: 'none'
  },
  motion: baseMotion,
  visualizer: {
    style: 'spectrum-bars',
    primary: '#F0A23B',
    secondary: '#5BE49B'
  }
};

export const winamp: Theme = {
  id: 'winamp',
  name: 'Winamp Retro',
  blurb: 'Detachable windows, metallic chrome, dense controls. Respectful, not parody.',
  color: {
    bg: '#1B1F1F',
    bgElevated: '#262B2B',
    panel: '#2E3434',
    border: '#080A0A',
    text: '#D9F0A6',
    textMuted: '#8AA66A',
    accent: '#9CFF3D',
    accentMuted: '#5F9F1F',
    indicator: '#FFCC00',
    danger: '#FF3838'
  },
  typography: {
    family: 'Tahoma, Geneva, sans-serif',
    familyDisplay: '"VT323", Tahoma, sans-serif',
    familyMono: '"VT323", "Courier New", monospace',
    weightBody: 400,
    weightDisplay: 500,
    tracking: '0'
  },
  surface: {
    radius: '2px',
    radiusLarge: '4px',
    chrome:
      'linear-gradient(180deg, #4A5050 0%, #2B3030 35%, #1A1F1F 36%, #2B3030 100%)',
    shadow:
      '0 0 0 1px #000, inset 0 1px 0 #6C7474, inset 0 -1px 0 #0A0C0C',
    backdrop: 'none'
  },
  motion: { ...baseMotion, fast: '60ms', medium: '120ms', slow: '200ms' },
  visualizer: {
    style: 'spectrum-bars',
    primary: '#FFCC00',
    secondary: '#FF3838'
  }
};

export const appleGlass: Theme = {
  id: 'apple-glass',
  name: 'Apple Glass',
  blurb: 'Glassmorphism, blurred panels, oversized type, floating controls.',
  color: {
    bg: '#0A0A14',
    bgElevated: 'rgba(255,255,255,0.06)',
    panel: 'rgba(255,255,255,0.08)',
    border: 'rgba(255,255,255,0.12)',
    text: '#FFFFFF',
    textMuted: 'rgba(255,255,255,0.6)',
    accent: '#7CA8FF',
    accentMuted: '#5276C2',
    indicator: '#9CFFB7',
    danger: '#FF6B81'
  },
  typography: {
    family:
      '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif',
    familyDisplay: '"SF Pro Display", -apple-system, sans-serif',
    familyMono: '"SF Mono", Menlo, monospace',
    weightBody: 400,
    weightDisplay: 600,
    tracking: '-0.01em'
  },
  surface: {
    radius: '16px',
    radiusLarge: '28px',
    chrome: 'linear-gradient(135deg, rgba(255,255,255,0.10), rgba(255,255,255,0.02))',
    shadow:
      '0 1px 0 rgba(255,255,255,0.18) inset, 0 20px 60px rgba(0,0,0,0.45)',
    backdrop: 'blur(24px) saturate(160%)'
  },
  motion: { ...baseMotion, slow: '600ms' },
  visualizer: {
    style: 'oscilloscope',
    primary: '#7CA8FF',
    secondary: '#FFFFFF'
  }
};

export const spotify: Theme = {
  id: 'spotify',
  name: 'Spotify Dark',
  blurb: 'High contrast, large artwork, strong typography.',
  color: {
    bg: '#000000',
    bgElevated: '#121212',
    panel: '#181818',
    border: '#272727',
    text: '#FFFFFF',
    textMuted: '#B3B3B3',
    accent: '#1DB954',
    accentMuted: '#149C44',
    indicator: '#1DB954',
    danger: '#E22134'
  },
  typography: {
    family: '"Circular", "Helvetica Neue", Helvetica, Arial, sans-serif',
    familyDisplay: '"Circular Display", "Helvetica Neue", sans-serif',
    familyMono: 'ui-monospace, "SF Mono", Menlo, monospace',
    weightBody: 400,
    weightDisplay: 700,
    tracking: '-0.01em'
  },
  surface: {
    radius: '8px',
    radiusLarge: '12px',
    chrome: 'linear-gradient(180deg, #1F1F1F 0%, #121212 100%)',
    shadow: '0 8px 24px rgba(0,0,0,0.6)',
    backdrop: 'none'
  },
  motion: baseMotion,
  visualizer: {
    style: 'spectrum-bars',
    primary: '#1DB954',
    secondary: '#FFFFFF'
  }
};

export const cyberdeck: Theme = {
  id: 'cyberdeck',
  name: 'Cyberdeck',
  blurb: 'Phosphor green, topology-heavy, diagnostics-forward.',
  color: {
    bg: '#020803',
    bgElevated: '#04120A',
    panel: '#061B0F',
    border: '#0F3A1E',
    text: '#39FF14',
    textMuted: '#1E8C25',
    accent: '#39FF14',
    accentMuted: '#1E8C25',
    indicator: '#A6FF00',
    danger: '#FF3232'
  },
  typography: {
    family: '"JetBrains Mono", "Fira Code", monospace',
    familyDisplay: '"JetBrains Mono", monospace',
    familyMono: '"JetBrains Mono", monospace',
    weightBody: 400,
    weightDisplay: 600,
    tracking: '0.04em'
  },
  surface: {
    radius: '0px',
    radiusLarge: '2px',
    chrome: 'linear-gradient(180deg, #061B0F 0%, #020803 100%)',
    shadow: '0 0 24px rgba(57, 255, 20, 0.08) inset',
    backdrop: 'none'
  },
  motion: { ...baseMotion, fast: '40ms', medium: '90ms', slow: '160ms' },
  visualizer: {
    style: 'topology',
    primary: '#39FF14',
    secondary: '#A6FF00'
  }
};

export const ambient: Theme = {
  id: 'ambient',
  name: 'Ambient',
  blurb: 'Soft gradients, minimal controls, low-motion. For wall panels.',
  color: {
    bg: '#101418',
    bgElevated: '#181D22',
    panel: '#1E242A',
    border: '#252C33',
    text: '#E8E5DC',
    textMuted: '#9AA1A8',
    accent: '#C9A37B',
    accentMuted: '#7E6650',
    indicator: '#A9C9B0',
    danger: '#D17B7B'
  },
  typography: {
    family: '"Cormorant Garamond", "Iowan Old Style", Georgia, serif',
    familyDisplay: '"Cormorant Garamond", serif',
    familyMono: '"JetBrains Mono", monospace',
    weightBody: 400,
    weightDisplay: 300,
    tracking: '0.03em'
  },
  surface: {
    radius: '14px',
    radiusLarge: '24px',
    chrome:
      'radial-gradient(120% 80% at 20% 0%, rgba(201,163,123,0.08), transparent 60%), linear-gradient(180deg, #1E242A 0%, #161A1F 100%)',
    shadow: '0 24px 60px rgba(0,0,0,0.45)',
    backdrop: 'blur(12px)'
  },
  motion: { fast: '320ms', medium: '600ms', slow: '1200ms', easing: 'ease-in-out' },
  visualizer: {
    style: 'ambient-flow',
    primary: '#C9A37B',
    secondary: '#A9C9B0'
  }
};

export const themes: Record<ThemeId, Theme> = {
  'analog-core': analogCore,
  winamp,
  'apple-glass': appleGlass,
  spotify,
  cyberdeck,
  ambient
};

export const themeList: Theme[] = [
  analogCore,
  winamp,
  appleGlass,
  spotify,
  cyberdeck,
  ambient
];

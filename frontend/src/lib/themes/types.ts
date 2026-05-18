export type ThemeId =
  | 'analog-core'
  | 'winamp'
  | 'apple-glass'
  | 'spotify'
  | 'cyberdeck'
  | 'ambient';

export type VisualizerStyle =
  | 'spectrum-bars'
  | 'oscilloscope'
  | 'topology'
  | 'waveform-grid'
  | 'ambient-flow';

export interface Theme {
  id: ThemeId;
  name: string;
  blurb: string;
  color: {
    bg: string;
    bgElevated: string;
    panel: string;
    border: string;
    text: string;
    textMuted: string;
    accent: string;
    accentMuted: string;
    indicator: string;
    danger: string;
  };
  typography: {
    family: string;
    familyDisplay: string;
    familyMono: string;
    weightBody: number;
    weightDisplay: number;
    tracking: string;
  };
  surface: {
    radius: string;
    radiusLarge: string;
    chrome: string;
    shadow: string;
    backdrop: string;
  };
  motion: {
    fast: string;
    medium: string;
    slow: string;
    easing: string;
  };
  visualizer: {
    style: VisualizerStyle;
    primary: string;
    secondary: string;
  };
}

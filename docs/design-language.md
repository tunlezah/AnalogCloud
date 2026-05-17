# Analog Cloud — Design Language

This document defines how Analog Cloud looks and feels. It is not
optional decoration — the visual identity is part of the product.

## 1. References

The product feels like a blend of:

- **Braun** — disciplined typography, restrained color, tactile knobs
- **Teenage Engineering** — playful precision, considered iconography
- **Ubiquiti UniFi** — topology and observability as first-class UI
- **Winamp** — dense, audio-nerd, character-driven
- **Apple Music / Spotify** — modern consumer polish where it helps
- **Modular synth patch bays** — explicit signal flow

It must never feel like:

- a generic enterprise dashboard
- a Bootstrap admin template
- a music library browser

## 2. Layout grammar

```
┌─────────────────────────────────────────────────────┐
│ TOP BAR — logo · input · output · EQ · theme        │
├──────────────┬──────────────────────────────────────┤
│ LEFT PANEL   │ MAIN CONTENT                         │
│              │                                      │
│   Inputs     │   Waveform                           │
│   Outputs    │   Visualizer                         │
│   Sessions   │   Playback controls                  │
│   Themes     │   Metadata                           │
│   Settings   │   Output status                      │
│              │                                      │
├──────────────┴──────────────────────────────────────┤
│ BOTTOM STATUS BAR — latency · codec · bitrate · ok  │
└─────────────────────────────────────────────────────┘
```

The layout is **shared across all themes**. Themes change typography,
chrome, density, animation, and the visualizer flavor — never the
information architecture.

## 3. Required themes

### 3.1 Analog Core (default, flagship)

| token              | value                                           |
|--------------------|-------------------------------------------------|
| background         | `#0B0B0D` matte charcoal                        |
| panel              | brushed dark metal (subtle linear-gradient)     |
| accent             | `#F0A23B` amber                                 |
| indicator          | `#5BE49B` subtle CRT green                      |
| typography         | Inter / Söhne, thin weights, generous tracking  |
| controls           | tactile, weighted, soft drop shadows            |
| glow               | warm, low-intensity                             |

This is the **most important theme**. Everything else flexes from here.

### 3.2 Winamp Retro

- Detachable window aesthetic (windows snap together)
- Metallic gradient panel chrome with hard 1px borders
- Dense control rows, condensed monospaced labels
- Classic horizontal spectrum analyzer (yellow→red bars)
- Modernized, never parody

### 3.3 Apple Glass

- Glassmorphism panels (`backdrop-filter: blur(24px)`)
- Soft pastel gradients on background
- Oversized SF-Pro-ish typography
- Floating controls with subtle shadows
- Plenty of whitespace

### 3.4 Spotify Dark

- High contrast, near-black background
- Large artwork in main column
- Strong heavy display type
- Playlist-style left rail of inputs/outputs

### 3.5 Cyberdeck

- Phosphor green (`#39FF14`) on near-black
- Topology graph dominates main area
- Diagnostic overlays (latency, jitter, buffer fill)
- Transport metrics always visible
- Waveform grids and crosshair cursors

### 3.6 Ambient

- Soft animated gradient background
- Minimal controls, hidden until hover
- Low-motion, slow easing
- Typography-led layout
- Designed for tablets, wall panels, always-on displays

## 4. Tokens

Every theme exports the same token set. Components consume tokens, never
raw colors.

```ts
type Theme = {
  id: 'analog-core' | 'winamp' | 'apple-glass' | 'spotify' | 'cyberdeck' | 'ambient';
  name: string;
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
    chrome: string;       // CSS for panel chrome (gradient/glass/etc)
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
    style: 'spectrum-bars' | 'oscilloscope' | 'topology' | 'waveform-grid' | 'ambient-flow';
    primary: string;
    secondary: string;
  };
};
```

## 5. Input cards

Every input card shows:

- source icon
- source name
- transport type (Bluetooth A2DP, line-in, …)
- codec
- sample rate
- live signal level meter (peak + RMS)
- mini waveform (last 5 s)
- reconnect state
- selected badge

## 6. Output cards

Every output card shows:

- output icon (browser / speaker / cast / airplay)
- latency estimate
- transport type
- connection state (idle / negotiating / streaming / reconnecting)
- active badge
- reconnect capability
- **Take Over Output** button

## 7. Visualizers

Required:

- FFT spectrum (1024-bin, log-frequency)
- Waveform (4096 samples, anti-aliased)
- Stereo VU meters (peak + RMS, ballistics per IEC 60268-17)

Preferred rendering: Canvas or WebGL. Animation must be smooth at 60 Hz.

## 8. Global Equalizer

10 bands at:

```
32  64  125  250  500  1k  2k  4k  8k  16k  Hz
```

Presets: Flat · Vinyl Warmth · Bass Focus · Speech · Tape · Night.

Sliders are large and tactile. A live frequency-response curve is drawn
behind the sliders.

## 9. Anti-patterns

- Tiny generic EQ sliders
- Bootstrap card grids
- Enterprise-style table layouts
- Per-output color schemes (themes are global)
- Treating themes as "skins" with only color changes

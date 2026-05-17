# Analog Cloud — Frontend

SvelteKit + TypeScript browser cockpit for Analog Cloud.

## Run

```bash
pnpm install
pnpm dev
```

The dev server runs on `http://127.0.0.1:5173` and proxies `/api` to the
backend at `127.0.0.1:7777`. Start the backend separately:

```bash
cd ../backend && cargo run -p analog-cloud-server
```

## Layout

```
src/
├── app.html, app.css      # shell + tokens
├── routes/                # SvelteKit pages
│   ├── +layout.svelte     # top bar / left panel / status bar shell
│   ├── +page.svelte       # main listening console
│   ├── eq/+page.svelte    # global 10-band EQ
│   ├── themes/+page.svelte
│   ├── outputs/+page.svelte
│   ├── sessions/+page.svelte
│   └── settings/+page.svelte
├── lib/
│   ├── api/               # backend client + event types
│   ├── components/        # TopBar, LeftPanel, StatusBar, cards
│   ├── eq/                # Equalizer
│   ├── session_ui/        # SessionPanel
│   ├── stores/            # svelte stores (session, levels)
│   ├── themes/            # theme engine
│   └── visualizers/       # Spectrum, VU, Sparkline
└── static/                # logo.png, icon.png
```

## Theme engine

Themes are NOT just color palettes — they alter typography, surface
chrome, density, motion and visualizer style. See
[`src/lib/themes/themes.ts`](src/lib/themes/themes.ts).

Every UI component consumes tokens (CSS custom properties) rather than
raw colors. The active theme is persisted to `localStorage` under
`analog-cloud:theme` and applied to `:root` on every change.

## Detachable cockpit

The frontend never owns playback. Closing the tab must not stop
Chromecast / AirPlay / local sessions. The browser **may** be a
playback target via WebRTC, but only as an output the user opts into.

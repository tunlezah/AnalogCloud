# Analog Cloud

> **Physical audio. Rebroadcast beautifully.**

Analog Cloud is a local-first realtime audio routing appliance. It ingests
physical and wireless audio sources (Bluetooth turntables, cassette decks,
line-in) into a Linux audio graph, normalizes them, and streams to one
active output destination at a time — browser (WebRTC), local speakers
(PipeWire), Chromecast (Opus/AAC over HTTP), or AirPlay (RAOP).

The system continues operating independently of the browser UI. The
browser is a **detachable control surface** and **optional playback
client** — never the playback authority.

## Architecture at a glance

```
┌──────────────────────────────────────────────────────────────┐
│                       SvelteKit UI                           │
│   detachable cockpit · WebRTC client · theme engine          │
└─────────────────────────────┬────────────────────────────────┘
                              │ HTTP + WebSocket
┌─────────────────────────────┴────────────────────────────────┐
│                     Rust backend (axum)                      │
│   api · session_manager · device_discovery · metadata · hass │
└─────────────────────────────┬────────────────────────────────┘
                              │ control + canonical PCM
┌─────────────────────────────┴────────────────────────────────┐
│                       Media engine                           │
│        PipeWire · WirePlumber · GStreamer · FFmpeg           │
│            Bluetooth · line-in · DSP · transcode             │
└──────────────────────────────────────────────────────────────┘
```

All audio is normalized at the boundary to **PCM · 48 kHz · stereo · float32**.
Resampling happens centrally — never chained.

## Critical rules

1. **Backend owns playback.** The browser never directly owns streams.
2. **One active output at a time.** Browser, local, Chromecast, or AirPlay.
3. **Canonical format.** PCM 48 kHz stereo float32 inside the graph.
4. **Offline-first.** Full operation with zero internet.
5. **Browser is detachable.** Closing the browser must not stop Chromecast,
   AirPlay, or local playback.

## Repo layout

```
analog-cloud/
├── backend/          # Rust workspace (axum + tokio)
│   ├── api/
│   ├── session_manager/
│   ├── device_discovery/
│   ├── metadata/
│   ├── hass/
│   └── settings/
│
├── media-engine/     # PipeWire / GStreamer / FFmpeg wrappers
│   ├── pipewire/
│   ├── gstreamer/
│   ├── transcoding/
│   ├── dsp/
│   └── outputs/
│
├── frontend/         # SvelteKit + TypeScript
│   ├── themes/
│   ├── components/
│   ├── visualizers/
│   ├── eq/
│   └── session_ui/
│
├── shared/           # Cross-language schemas + event models
│   ├── schemas/
│   └── event_models/
│
└── docs/             # Architecture, specs, runbooks
```

## Host platform

Target: **Ubuntu 26.04 LTS** on an Intel Mac Mini.

## Getting started

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable

# System audio stack (Ubuntu 26.04)
sudo apt install -y \
  pipewire pipewire-pulse pipewire-audio-client-libraries wireplumber \
  libpipewire-0.3-dev libspa-0.2-dev \
  gstreamer1.0-tools gstreamer1.0-plugins-{base,good,bad,ugly} \
  gstreamer1.0-libav libgstreamer1.0-dev \
  ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswresample-dev \
  pkg-config libssl-dev build-essential

# Node toolchain
corepack enable
corepack prepare pnpm@latest --activate
```

### Run the backend

```bash
cd backend
cargo run -p analog-cloud-server
```

The API listens on `http://127.0.0.1:7777` by default.

### Run the frontend

```bash
cd frontend
pnpm install
pnpm dev
```

The UI is served on `http://127.0.0.1:5173` and talks to the backend on `7777`.

## Implementation phases

- **Phase 1** — Bluetooth + line-in ingest, local + browser playback,
  topology UI, EQ, themes.
- **Phase 2** — Chromecast, AirPlay, reconnectable sessions, metadata
  enrichment.
- **Phase 3** — Home Assistant integration, advanced visualizers, DSP
  presets, waveform indexing.
- **Phase 4** — Recording, archival, playback history, advanced metadata.

See [`docs/architecture.md`](docs/architecture.md) for the full design.

## Branding

The cloud with the embedded waveform and the silver record sleeve is the
canonical logo. Matte charcoal background, brushed dark metal cloud,
amber accent. **Do not redesign.**

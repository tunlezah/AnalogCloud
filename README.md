# Analog Cloud

> **Physical audio. Rebroadcast beautifully.**

Analog Cloud is a **local-first realtime audio routing appliance**. It
ingests physical and wireless audio sources — Bluetooth turntables,
cassette decks, line-in — into a Linux audio graph, normalizes them, and
streams to one active output destination at a time: a browser (WebRTC),
local speakers (PipeWire), a Chromecast (Opus/AAC over HTTP), or an
AirPlay receiver (RAOP).

The system runs continuously on the host. The browser UI is a
**detachable control surface** and an **optional playback client** — it
is never the playback authority. Closing the tab does not stop a
Chromecast or AirPlay session.

```
┌──────────────────────────────────────────────────────────────┐
│                       SvelteKit UI                           │
│   detachable cockpit · WebRTC client · theme engine          │
└─────────────────────────────┬────────────────────────────────┘
                              │ HTTP + WebSocket + SSE
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

All audio inside the graph is normalized at the boundary to
**PCM · 48 kHz · stereo · float32**, interleaved. **Resampling happens
exactly once**, at ingest — output transcoders consume canonical PCM
directly.

---

## Table of contents

- [What it does](#what-it-does)
- [How it works](#how-it-works)
- [Quick start (recommended)](#quick-start-recommended)
- [The installer in depth](#the-installer-in-depth)
- [Manual install](#manual-install)
- [Running it](#running-it)
- [Configuration](#configuration)
- [Project layout](#project-layout)
- [Implementation phases](#implementation-phases)
- [Troubleshooting](#troubleshooting)

---

## What it does

Analog Cloud reframes analog and physical audio sources as first-class
network citizens, and exposes them through a premium listening console.

- **Captures** audio from Bluetooth A2DP sources, USB line-in, soundcards.
- **Normalizes** everything to a single internal format (48 kHz / float32
  / stereo) so the rest of the system never has to think about formats.
- **Routes** the active source to exactly one destination at a time —
  browser, local speakers, Chromecast, or AirPlay.
- **Transcodes** on the fly for network sinks (Opus to browsers and
  Chromecast, RAOP to AirPlay) without ever re-resampling.
- **Surveils** the signal — VU meters, spectrum, waveform snapshots,
  level metering, latency / buffer telemetry at 5 Hz over SSE.
- **Enriches** metadata (optionally) via MusicBrainz + AcoustID.
- **Publishes** read-only state to Home Assistant over MQTT.
- **Persists** user settings and recordings under XDG paths.

## How it works

### Five rules the system never breaks

1. **The backend owns playback.** Browsers attach to sessions; they
   never own them.
2. **One active output at a time.** Switching destinations is an atomic
   takeover — the new session is constructed in `armed`, the old one is
   gracefully stopped, then the new one transitions to `playing`. There
   is never more than one session in `playing`.
3. **Canonical format inside the graph:** PCM 48 kHz stereo float32.
4. **Offline-first.** Every core feature works with no internet.
5. **Browser is detachable.** Closing the browser must not stop
   Chromecast, AirPlay, or local playback.

### Process model

| Component                | What it is                                                    |
|--------------------------|---------------------------------------------------------------|
| `analog-cloud-server`    | Single long-running daemon. Axum HTTP/WS server, session and output ownership, settings, device discovery, metadata cache, HASS bridge. |
| `media-engine` (in-proc) | Speaks to PipeWire via `libpipewire` and to GStreamer via `gstreamer-rs`. Spawns helpers (e.g. `ffmpeg`) when in-process pipelines aren't appropriate. |
| Frontend                 | SvelteKit SPA. In dev it runs under Vite and proxies API/WS/SSE to the backend; in production it is built and served as static assets by the backend. |

### Outputs

| Output     | Transport            | Latency target | Ownership | Notes |
|------------|----------------------|----------------|-----------|-------|
| Browser    | WebRTC (Opus)        | 80–200 ms      | Browser   | Ephemeral. Dies with the tab. |
| Local      | PipeWire sink        | 20–60 ms       | Backend   | Persistent. |
| Chromecast | HTTP pull (Opus/AAC) | 1.0–2.0 s      | Backend   | Cast device pulls the URL — we never push raw PCM. |
| AirPlay    | RAOP                 | 1.5–2.5 s      | Backend   | Reconnectable. |

### Realtime rules

Realtime threads (PipeWire callbacks, GStreamer fill callbacks, anything
on SCHED_FIFO) must never block on disk, databases, metadata lookups, or
network calls, and must not allocate on the hot path. All such work is
deferred to non-realtime workers over MPSC channels.

### HTTP surface (selected)

| Endpoint                | Purpose                                  |
|-------------------------|------------------------------------------|
| `GET  /api/inputs`      | List discovered inputs.                  |
| `GET  /api/outputs`     | List candidate outputs.                  |
| `GET  /api/sessions`    | Snapshot of session state.               |
| `POST /api/sessions`    | Arm + start a session (atomic takeover). |
| `GET  /api/settings`    | Current settings (EQ, theme, etc.).      |
| `POST /api/settings`    | Persist a settings mutation.             |
| `GET  /events`          | Server-Sent Events: level samples, session state, device events. |
| `GET  /metrics`         | Prometheus scrape endpoint.              |
| `GET  /state`           | Debug snapshot of internal state.        |

---

## Quick start (recommended)

```bash
git clone <repo-url> analog-cloud
cd analog-cloud
./scripts/install.sh
./scripts/run.sh        # starts backend + frontend together
```

`install.sh` is interactive: it probes your toolchain, picks ports that
don't conflict with anything else on the box, and only builds what
needs building.

For unattended runs:

```bash
./scripts/install.sh --force --backend-port 7777 --frontend-port 5173
```

---

## The installer in depth

The installer is the canonical way to set up the project on a host. It
lives in `scripts/install.sh` and is designed around three commitments:

### 1. Port conflicts are handled, not assumed away

The backend wants `127.0.0.1:7777` by default and the Vite dev server
wants `127.0.0.1:5173`. Either of those can be in use — by a previous
run of this app, by an unrelated dev server, by `nginx`, by anything.

The installer probes each port with `ss` (with fallbacks to `lsof`,
`netstat`, and `/proc/net/tcp` so we never assume a probe tool exists),
and if a port is occupied it:

- shows the user the **pid and command name** holding the port,
- searches forward for a free port (skipping a small set of
  well-known-but-busy alternatives like `8080`, `8000`, `3000`),
- offers that as the new default,
- accepts any port the user picks, then re-probes — so even if the
  user's choice is also taken, they get the same flow rather than a
  cryptic bind error five minutes later when `cargo run` finally tries
  to listen.

You can also force a port from the command line:

```bash
./scripts/install.sh --backend-port 18080 --frontend-port 15173
```

If the forced port is busy, the installer refuses to proceed and prints
who is holding it.

### 2. Reinstalls are first-class

The installer never *infers* "this is a reinstall" from a `target/`
directory being present — those can exist for unrelated reasons. Instead
it writes a marker file:

```
$XDG_CONFIG_HOME/analog-cloud/installer.env
```

…and consults it on every run. The marker remembers your last chosen
ports, what was built, and whether you opted into the system audio
stack. If the marker is present, you see your previous configuration and
are asked whether to update in place or wipe and reinstall. If the
marker is **missing** but build artifacts exist, the installer surfaces
that mismatch (`"build artifacts present, no marker"`) and lets you
decide rather than silently doing the wrong thing.

```bash
./scripts/install.sh --reinstall     # wipe target/, node_modules/, build/, .svelte-kit/
./scripts/uninstall.sh               # remove artifacts + marker
./scripts/uninstall.sh --purge       # …and wipe user config + recordings
```

### 3. Assume nothing about the toolchain

The installer probes for `cargo`, `rustc`, `node` (≥ 18), and `pnpm`
fresh on every run. If anything is missing it offers — never silently —
to install it via the host package manager:

| OS                  | Path used                                       |
|---------------------|-------------------------------------------------|
| Ubuntu / Debian     | `apt` for system libs + NodeSource for Node.js  |
| Fedora              | `dnf`                                           |
| Arch / Manjaro      | `pacman`                                        |
| macOS               | Homebrew (informational — PipeWire is Linux-only; backend runs in mock mode) |
| Anything else       | Best-effort detection, manual instructions printed |

For the audio stack (PipeWire, WirePlumber, GStreamer, FFmpeg, headers)
the installer treats it as **optional**: the workspace also builds in a
mock mode without those libraries present, useful for UI development on
a non-target machine. Skip the OS prompt entirely with:

```bash
./scripts/install.sh --skip-system
```

### Installer flags

| Flag                       | Behavior                                                |
|----------------------------|---------------------------------------------------------|
| `--reinstall`              | Wipe build artifacts before rebuilding.                 |
| `--force`                  | Accept all defaults; implies non-interactive.           |
| `--skip-system`            | Don't probe or install OS-level audio deps.             |
| `--skip-build`             | Configure ports + write `.env` only; don't build.       |
| `--backend-port  <N>`      | Pin the backend port (fail-fast if busy).               |
| `--frontend-port <N>`      | Pin the frontend dev port.                              |
| `--release`                | Build the backend with `cargo build --release`.         |
| `--help`                   | Print usage.                                            |

The installer also respects `ANALOG_CLOUD_NONINTERACTIVE=1` (set
automatically by `--force`), `NO_COLOR=1`, and `XDG_CONFIG_HOME`.

---

## Manual install

If you'd rather wire it up yourself:

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System audio stack (Ubuntu 26.04 — pick the appropriate distro)
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

# Build
cargo build -p analog-cloud-server
( cd frontend && pnpm install && pnpm build )
```

---

## Running it

After `scripts/install.sh` writes `.env`, either component is launchable
with the ports the installer chose:

```bash
# Both at once, in dev mode (Ctrl-C kills both)
./scripts/run.sh

# Or piecewise
set -a; . ./.env; set +a
cargo run -p analog-cloud-server          # backend
( cd frontend && pnpm dev )               # frontend
```

`run.sh` exists purely for convenience; in production you'd wrap
`analog-cloud-server` in a systemd service and serve the built frontend
from the backend's static handler.

---

## Configuration

| Where                                                     | What lives there                                            |
|-----------------------------------------------------------|-------------------------------------------------------------|
| `./.env` (repo root)                                      | Generated by the installer. Backend bind address + frontend port. Hand-edits to other keys are preserved on re-run. |
| `$XDG_CONFIG_HOME/analog-cloud/config.toml`               | Runtime settings: EQ, theme, enrichment mode, HASS settings. Owned by the `settings` crate. |
| `$XDG_CONFIG_HOME/analog-cloud/installer.env`             | Installer marker — last-run ports, build timestamps, whether audio deps were installed. Safe to delete; the installer will rebuild it. |
| `$XDG_DATA_HOME/analog-cloud/`                            | Recordings, waveform snapshots, persisted media.            |

### Environment variables read by the runtime

| Variable                    | Read by    | Default            |
|-----------------------------|------------|--------------------|
| `ANALOG_CLOUD_BIND`         | backend    | `127.0.0.1:7777`   |
| `RUST_LOG`                  | backend    | `info,analog_cloud=debug` |
| `VITE_BACKEND_URL`          | frontend   | derived from `ANALOG_CLOUD_BIND` |
| `VITE_FRONTEND_PORT`        | frontend   | `5173`             |

---

## Project layout

```
analog-cloud/
├── backend/                       # Rust workspace (axum + tokio)
│   ├── analog-cloud-server/       # daemon entry point
│   ├── api/                       # HTTP/WS routes
│   ├── session_manager/           # session lifecycle, takeover semantics
│   ├── device_discovery/          # mDNS, Bluetooth, line-in discovery
│   ├── metadata/                  # MusicBrainz + AcoustID
│   ├── hass/                      # Home Assistant MQTT bridge
│   └── settings/                  # TOML-backed settings store
│
├── media-engine/                  # PipeWire / GStreamer / FFmpeg wrappers
│
├── frontend/                      # SvelteKit + TypeScript cockpit
│   ├── src/lib/components/        # TopBar, StatusBar, InputCard, ...
│   ├── src/lib/visualizers/       # VuMeter, Spectrum, Sparkline
│   ├── src/lib/api/               # client, events (SSE), webrtc, types
│   └── src/routes/                # /, /sessions, /outputs, /eq, /settings
│
├── shared/                        # cross-language schemas + event models
├── docs/                          # architecture, design, phase plan
└── scripts/                       # installer + runtime helpers
    ├── install.sh
    ├── uninstall.sh
    ├── run.sh
    └── lib/                       # common, ports, deps, state helpers
```

---

## Implementation phases

| Phase | Scope                                                                       |
|-------|-----------------------------------------------------------------------------|
| 1     | Bluetooth + line-in ingest, local + browser playback, topology UI, EQ, themes |
| 2     | Chromecast, AirPlay, reconnectable sessions, metadata enrichment            |
| 3     | Home Assistant integration, advanced visualizers, DSP presets, waveform indexing |
| 4     | Recording, archival, playback history, advanced metadata                    |

See [`docs/architecture.md`](docs/architecture.md) for the full design
and [`docs/phases.md`](docs/phases.md) for the scope cuts inside each
phase.

---

## Troubleshooting

**`bind: address already in use` when `cargo run`.**
Something is on your chosen port. Re-run `./scripts/install.sh`; it will
detect the conflict and offer a free port. Or pin one explicitly with
`--backend-port`.

**The installer says "build artifacts present, no marker".**
You built the project by hand at some point. That's fine — answer the
prompt to either reuse those artifacts (faster incremental rebuild) or
wipe them (clean reinstall).

**`cargo build` fails on PipeWire / GStreamer headers.**
The system audio stack is missing. Either re-run `./scripts/install.sh`
and accept the "install audio stack" prompt, or install the dev packages
listed in [Manual install](#manual-install). For UI-only work you can
build without them by enabling the workspace's mock-mode features.

**The frontend can't reach the backend.**
Check that `VITE_BACKEND_URL` in `.env` matches the actual
`ANALOG_CLOUD_BIND`. The installer keeps these in sync; manual edits to
one without the other will desync the proxy.

**I want to start over completely.**
`./scripts/uninstall.sh --purge` removes build artifacts, the installer
marker, runtime config, and recordings.

---

## Branding

The cloud with the embedded waveform and the silver record sleeve is the
canonical logo. Matte charcoal background, brushed dark metal cloud,
amber accent. **Do not redesign.**

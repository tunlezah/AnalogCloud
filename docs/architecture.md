# Analog Cloud — Architecture

This document is the source of truth for how Analog Cloud is structured.
It expands the Master Build Specification.

## 1. Identity

Analog Cloud is a **local-first audio infrastructure appliance**, not a
music library or a streaming app. The product reframes analog sources as
first-class network audio citizens and exposes them through a premium
listening console.

The browser is a **detachable control cockpit** — never the playback
authority.

## 2. Top-level architecture

```
SvelteKit UI  ── HTTP/WS ──▶  Rust backend (axum)  ── ctrl ──▶  Media engine
                                                                  │
                                                                  ▼
                                                          PipeWire graph
```

### Process boundaries

- **`analog-cloud-server`** — single long-running daemon. Owns sessions,
  ownership of outputs, settings, discovery, metadata cache, and HASS
  bridge.
- **Media engine** — in-process library inside `analog-cloud-server`,
  speaking to PipeWire via `libpipewire` and to GStreamer pipelines via
  `gstreamer-rs`. May spawn helper processes (e.g. `ffmpeg` for
  transcoding) when in-process pipelines are not appropriate.
- **Frontend** — SvelteKit SPA + SSR. In production it is served by the
  backend as static assets; in dev it runs on Vite and proxies API calls.

## 3. Canonical audio format

All audio inside the graph is normalized at the source boundary to:

| field        | value              |
|--------------|--------------------|
| sample format| `float32`          |
| sample rate  | `48000 Hz`         |
| channels     | `stereo (2)`       |
| layout       | interleaved        |
| dither       | TPDF on output     |

**Resampling is performed exactly once**, at the ingest boundary, by a
single high-quality SoX/SRC resampler. Output paths must never resample
again — output transcoders consume the canonical format directly.

## 4. Session model

A **session** is a backend-owned playback instance. It binds exactly one
input source to exactly one output destination via a transport. Clients
(browsers) attach to sessions; clients never own them.

```jsonc
{
  "session_id": "01HW5R…",
  "input_id": "soundburger_bt",
  "output_type": "chromecast",
  "output_device": "Living Room",
  "transport": "opus_http",
  "started_at": "2026-05-17T12:34:56Z",
  "controller_clients": ["browser-uuid-1"],
  "state": "playing",
  "stats": {
    "latency_ms": 320,
    "buffer_ms": 1200,
    "codec": "opus@128k",
    "reconnects": 0
  }
}
```

### Lifecycle

```
   ┌────────┐  select  ┌────────┐  start  ┌─────────┐
   │ idle   ├─────────▶│ armed  ├────────▶│ playing │
   └────────┘          └────────┘         └────┬────┘
                                               │ stop / takeover
                                               ▼
                                         ┌──────────┐
                                         │ stopped  │
                                         └──────────┘
```

A "takeover" replaces the current session atomically: the new session is
constructed in `armed`, the old session is gracefully stopped, then the
new session transitions to `playing`. There is **never** more than one
session in `playing` state at the same time.

## 5. Outputs

| Output     | Transport         | Latency target | Ownership | Notes |
|------------|-------------------|----------------|-----------|-------|
| Browser    | WebRTC (Opus)     | 80–200 ms      | Browser   | Ephemeral. Dies with the tab. |
| Local      | PipeWire sink     | 20–60 ms       | Backend   | Persistent. |
| Chromecast | HTTP pull (Opus/AAC) | 1.0–2.0 s    | Backend   | Cast device **pulls** the URL. |
| AirPlay    | RAOP              | 1.5–2.5 s      | Backend   | Reconnectable. |

### Chromecast

The backend hosts an HTTP endpoint that serves a continuous Opus-in-WebM
or AAC-in-ADTS stream. We launch the Cast app pointing it at our own URL
and let it pull. We **never** push raw PCM.

### AirPlay

RAOP-compatible. Backend uses an embedded RAOP sender. Higher latency
is acceptable; the spec calls this out explicitly.

## 6. Realtime rules

Realtime threads (PipeWire processing callbacks, GStreamer fill
callbacks, anything on an SCHED_FIFO/real-time priority) must never:

- block on disk IO
- block on databases or caches
- block on metadata lookups
- block on network calls
- allocate via the system allocator on the hot path (preallocate)

All such work is sent through an MPSC channel to a non-realtime worker.

## 7. Online enrichment

Provider: **MusicBrainz + AcoustID**.

Modes:

- **Off** — never fingerprint.
- **Immediate** — fingerprint on input lock.
- **Delayed** — fingerprint after `wait_seconds` of stable signal.
- **Manual** — only on user request.

Enrichment never affects playback. Metadata is purely informational.

## 8. Home Assistant

HASS is **read-only**. It receives state via MQTT and may display:

- active input
- active output
- playback state
- track metadata
- volume
- artwork URL
- waveform snapshot (PNG, periodic)

HASS **must not** be allowed to route audio. Routing decisions live in
the backend only.

## 9. Settings + persistence

Settings live in a single TOML file at
`$XDG_CONFIG_HOME/analog-cloud/config.toml`. Recordings and waveform
snapshots live in `$XDG_DATA_HOME/analog-cloud/`.

The settings module owns all reads/writes and emits change events on the
internal event bus.

## 10. Observability

The backend exposes:

- a structured `tracing` log (configurable via `RUST_LOG`)
- a `/metrics` Prometheus endpoint
- a `/state` snapshot endpoint for debug UIs
- a `/events` Server-Sent Events stream for the frontend

Each session emits per-stat samples at 5 Hz to the SSE stream.

## 11. Phased implementation

| Phase | Scope                                                                 |
|-------|-----------------------------------------------------------------------|
| 1     | Bluetooth + line-in ingest, local + browser playback, EQ, themes      |
| 2     | Chromecast, AirPlay, reconnectable sessions, metadata enrichment      |
| 3     | HASS integration, advanced visualizers, DSP presets, waveform index   |
| 4     | Recording, archival, playback history, advanced metadata              |

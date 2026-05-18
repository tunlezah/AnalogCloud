# Phased delivery plan

Analog Cloud ships in four phases. Each phase has a hard gate: a phase
is only complete when its end-to-end demo works on the target Mac Mini
running Ubuntu 26.04 LTS.

## Phase 1 — Listening console

**Goal:** play a real Bluetooth turntable to local speakers and to a
browser, controlled from the SvelteKit UI.

- [ ] PipeWire ingest: enumerate sources, surface BT + line-in
- [ ] Canonical resampler at the ingest boundary (48 kHz / f32 / stereo)
- [ ] Local sink output
- [ ] Browser WebRTC output (offer/answer over `/api/webrtc`)
- [ ] Session manager: armed → playing, atomic takeover
- [ ] Topology UI: input cards + output cards + take-over button
- [ ] Global 10-band EQ (DSP + UI)
- [ ] Theme engine: Analog Core + all five additional themes

**Demo:** spin the Sound Burger, hit "This Browser", hit "Local
Speakers", swap themes, tweak EQ.

## Phase 2 — Reach

**Goal:** play to Chromecast and AirPlay, reconnect cleanly when
sessions drop.

- [ ] mDNS discovery (`_googlecast._tcp`, `_raop._tcp`)
- [ ] Cast pull-server: HTTP endpoint serving Opus-in-WebM
- [ ] Cast launcher: send LAUNCH + LOAD to the device
- [ ] AirPlay RAOP sender
- [ ] Reconnectable sessions: state survives transport drops
- [ ] AcoustID fingerprint + MusicBrainz resolve (delayed mode default)
- [ ] Settings UI for enrichment mode and audio mode

**Demo:** select an input, send it to a living-room Chromecast, close
the browser tab, reopen it, observe the session is still streaming and
the UI reconnects to it.

## Phase 3 — Integrations + polish

**Goal:** integrate into the home and become visually rich.

- [ ] Home Assistant MQTT publisher (informational only)
- [ ] Waveform indexing + snapshot artwork
- [ ] Advanced visualizers (topology view for Cyberdeck theme; ambient
      flow for Ambient theme)
- [ ] DSP presets per source kind (Vinyl Warmth on `bluetooth_turntable`)
- [ ] Latency / jitter / buffer-fill overlays
- [ ] Per-theme visualizer styles wired through

**Demo:** HASS dashboard reflects active input and artwork; Cyberdeck
theme shows topology graph; Ambient theme reduces motion.

## Phase 4 — Archive

**Goal:** keep what passed through the appliance.

- [ ] Recording (lossless FLAC into `$XDG_DATA_HOME/analog-cloud/`)
- [ ] Archival manifest (rolling DB of recorded segments)
- [ ] Playback history (input lock → fingerprint → outcome)
- [ ] Advanced metadata: relations, year, label, MBID provenance

**Demo:** browse 30 days of vinyl listening, drill into one session,
play back the recording, edit fingerprint result.

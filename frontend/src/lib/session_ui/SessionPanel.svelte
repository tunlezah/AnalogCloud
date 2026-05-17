<script lang="ts">
  import { onDestroy } from 'svelte';
  import { activeSession, inputs, outputs } from '$lib/stores/session';
  import { api } from '$lib/api/client';
  import { BrowserPlayback } from '$lib/api/webrtc';
  import VuMeter from '$lib/visualizers/VuMeter.svelte';
  import Spectrum from '$lib/visualizers/Spectrum.svelte';

  $: input = $inputs.find((i) => $activeSession && i.id === $activeSession.input_id);
  $: output = $outputs.find(
    (o) => $activeSession && o.kind === $activeSession.output_kind && o.name === $activeSession.output_device
  );

  let audioEl: HTMLAudioElement;
  let playback: BrowserPlayback | null = null;
  let webrtcError = '';

  $: maybeStartWebRtc($activeSession?.output_kind, $activeSession?.session_id);

  async function maybeStartWebRtc(kind: string | undefined, sessionId: string | undefined) {
    if (!audioEl) return;
    if (kind === 'browser' && sessionId) {
      if (playback) playback.stop();
      playback = new BrowserPlayback();
      webrtcError = '';
      try {
        await playback.start(audioEl);
      } catch (err) {
        webrtcError = err instanceof Error ? err.message : String(err);
        console.warn('webrtc start failed', err);
      }
    } else if (playback) {
      playback.stop();
      playback = null;
    }
  }

  onDestroy(() => {
    if (playback) playback.stop();
  });

  async function stop() {
    try {
      await api.stopSession();
    } catch (err) {
      console.warn('stop failed', err);
    }
  }
</script>

<section class="session panel">
  <header>
    <h2>Now Playing</h2>
    {#if $activeSession}
      <span class="badge" data-tone="live">{$activeSession.state}</span>
    {:else}
      <span class="badge">idle</span>
    {/if}
  </header>

  <Spectrum />

  <div class="grid">
    <div class="block">
      <span class="muted small">Input</span>
      <strong>{input?.name ?? '—'}</strong>
      <span class="muted small">
        {input?.codec ?? '—'} ·
        {input?.native_format
          ? `${(input.native_format.sample_rate / 1000).toFixed(1)} kHz`
          : '—'}
      </span>
    </div>
    <div class="block">
      <span class="muted small">Output</span>
      <strong>{output?.name ?? '—'}</strong>
      <span class="muted small">{$activeSession?.transport ?? '—'}</span>
    </div>
    <div class="block">
      <span class="muted small">Levels</span>
      <VuMeter />
    </div>
  </div>

  <div class="controls">
    <audio bind:this={audioEl} autoplay></audio>
    {#if webrtcError}
      <span class="muted small">webrtc: {webrtcError}</span>
    {/if}
    <button data-variant="ghost" disabled={!$activeSession} on:click={stop}>Stop</button>
  </div>
</section>

<style>
  .session {
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h2 {
    font-size: 1rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ac-text-muted);
  }

  .small {
    font-size: 0.65rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
  }

  .block {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.75rem 0.9rem;
    border-radius: var(--ac-radius);
    background: var(--ac-bg-elevated);
    border: 1px solid var(--ac-border);
  }

  .block strong { font-weight: 500; }

  .controls {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>

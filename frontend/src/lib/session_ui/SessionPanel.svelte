<script lang="ts">
  import { activeSession, inputs, outputs } from '$lib/stores/session';
  import { api } from '$lib/api/client';
  import VuMeter from '$lib/visualizers/VuMeter.svelte';
  import Spectrum from '$lib/visualizers/Spectrum.svelte';

  $: input = $inputs.find((i) => $activeSession && i.id === $activeSession.input_id);
  $: output = $outputs.find(
    (o) => $activeSession && o.kind === $activeSession.output_kind && o.name === $activeSession.output_device
  );

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

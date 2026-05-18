<script lang="ts">
  import type { OutputDescriptor } from '$lib/api/types';

  export let output: OutputDescriptor;
  export let active = false;

  const transportLabel: Record<OutputDescriptor['transport'], string> = {
    web_rtc_opus: 'WebRTC · Opus',
    pipewire_sink: 'PipeWire Sink',
    opus_http: 'HTTP · Opus',
    aac_http: 'HTTP · AAC',
    raop: 'RAOP'
  };

  const icon: Record<OutputDescriptor['kind'], string> = {
    browser: '⌬',
    local: '◉',
    chromecast: '▶',
    air_play: '✦'
  };

  $: latency = output.latency_ms_estimate;
</script>

<div class="card output" data-active={active}>
  <div class="row">
    <span class="icon" aria-hidden="true">{icon[output.kind]}</span>
    <div class="name">
      <strong>{output.name}</strong>
      <span class="muted small">{transportLabel[output.transport]}</span>
    </div>
    {#if active}<span class="badge" data-tone="accent">active</span>{/if}
  </div>

  <div class="meta">
    <span class="badge">{output.state}</span>
    {#if latency != null}<span class="badge">~{latency} ms</span>{/if}
    {#if output.reconnectable}<span class="badge">reconnectable</span>{/if}
  </div>

  <button data-variant={active ? 'ghost' : 'accent'} on:click>
    {active ? 'Active output' : 'Take Over Output'}
  </button>
</div>

<style>
  .output { gap: 0.65rem; }

  .row { display: flex; align-items: center; gap: 0.65rem; }

  .icon {
    font-size: 1.4rem;
    width: 1.6rem;
    text-align: center;
    color: var(--ac-accent);
  }

  .name {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
    flex: 1;
    min-width: 0;
  }

  .name strong { font-weight: 500; }

  .small {
    font-size: 0.65rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }
</style>

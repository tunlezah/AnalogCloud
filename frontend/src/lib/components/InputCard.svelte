<script lang="ts">
  import type { InputDescriptor } from '$lib/api/types';
  import Sparkline from '$lib/visualizers/Sparkline.svelte';

  export let input: InputDescriptor;
  export let selected = false;

  const transportLabel: Record<InputDescriptor['transport'], string> = {
    bluetooth_a2dp: 'Bluetooth A2DP',
    bluetooth_hfp: 'Bluetooth HFP',
    alsa: 'ALSA',
    pipewire_node: 'PipeWire',
    virtual: 'Virtual'
  };

  const icon: Record<InputDescriptor['kind'], string> = {
    bluetooth_turntable: '⊙',
    bluetooth_cassette: '▭',
    bluetooth_generic: '⌁',
    line_in: '◐',
    usb: '⎙',
    virtual: '◇'
  };

  $: levelPct = clampMeter(input.last_level_dbfs ?? -120);

  function clampMeter(dbfs: number): number {
    const min = -60;
    if (dbfs <= min) return 0;
    if (dbfs >= 0) return 100;
    return ((dbfs - min) / -min) * 100;
  }
</script>

<button class="card" data-active={selected} on:click>
  <div class="row">
    <span class="icon" aria-hidden="true">{icon[input.kind]}</span>
    <div class="name">
      <strong>{input.name}</strong>
      <span class="muted small">{transportLabel[input.transport]}</span>
    </div>
    {#if selected}<span class="badge" data-tone="accent">selected</span>{/if}
    {#if input.connected}<span class="badge" data-tone="live">live</span>{/if}
  </div>

  <div class="meta">
    {#if input.codec}<span class="badge">{input.codec}</span>{/if}
    {#if input.native_format}
      <span class="badge">
        {(input.native_format.sample_rate / 1000).toFixed(1)} kHz · {input.native_format.channels}ch · {input.native_format.format}
      </span>
    {/if}
  </div>

  <div class="meter">
    <span class="bar" style="--w: {levelPct}%" />
  </div>

  <Sparkline value={input.last_level_dbfs ?? -120} />
</button>

<style>
  .card { text-align: left; gap: 0.6rem; }

  .row {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .icon {
    font-size: 1.4rem;
    color: var(--ac-accent);
    width: 1.6rem;
    text-align: center;
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

  .meter {
    height: 6px;
    border-radius: 3px;
    background: var(--ac-bg-elevated);
    overflow: hidden;
    border: 1px solid var(--ac-border);
  }

  .bar {
    display: block;
    height: 100%;
    width: var(--w, 0%);
    background: linear-gradient(
      90deg,
      var(--ac-indicator) 0%,
      var(--ac-accent) 70%,
      var(--ac-danger) 100%
    );
    transition: width var(--ac-motion-fast) linear;
  }
</style>

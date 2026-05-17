<script lang="ts">
  import { EQ_BANDS_HZ } from '$lib/api/types';
  import type { EqPreset, EqualizerSettings } from '$lib/api/types';

  export let value: EqualizerSettings = {
    enabled: true,
    preset: 'flat',
    bands_db: new Array(10).fill(0)
  };

  const presets: Record<EqPreset, number[]> = {
    flat:           [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    'vinyl-warmth': [3, 2, 1, 0, -1, -1, 0, 1, 2, 1],
    'bass-focus':   [6, 5, 4, 2, 0, 0, 0, 0, 0, 0],
    speech:         [-3, -2, 0, 2, 4, 4, 3, 1, -1, -2],
    tape:           [2, 2, 1, 0, -1, -2, -1, 0, 2, 3],
    night:          [-2, -1, 0, 0, 1, 1, 0, -1, -3, -5],
    custom:         []
  };

  function applyPreset(name: EqPreset) {
    const next = presets[name];
    if (!next || next.length === 0) return;
    value = { ...value, preset: name, bands_db: [...next] };
  }

  function setBand(i: number, db: number) {
    const bands = [...value.bands_db];
    bands[i] = db;
    value = { ...value, preset: 'custom', bands_db: bands };
  }

  function onPresetClick(name: string) {
    applyPreset(name as EqPreset);
  }

  function onBandInput(i: number, e: Event) {
    const target = e.currentTarget as HTMLInputElement;
    setBand(i, Number(target.value));
  }

  function fmtHz(hz: number) {
    return hz >= 1000 ? `${hz / 1000}k` : `${hz}`;
  }
</script>

<section class="eq panel">
  <header>
    <h2>Global Equalizer</h2>
    <label class="toggle">
      <input type="checkbox" bind:checked={value.enabled} />
      <span>{value.enabled ? 'On' : 'Off'}</span>
    </label>
  </header>

  <div class="presets">
    {#each Object.keys(presets) as p}
      {#if p !== 'custom'}
        <button
          data-variant={value.preset === p ? 'accent' : 'ghost'}
          on:click={() => onPresetClick(p)}
        >
          {p.replace('-', ' ')}
        </button>
      {/if}
    {/each}
  </div>

  <div class="bands">
    {#each EQ_BANDS_HZ as hz, i}
      <div class="band">
        <span class="db mono">{value.bands_db[i].toFixed(1)} dB</span>
        <input
          type="range"
          min="-12"
          max="12"
          step="0.5"
          value={value.bands_db[i]}
          on:input={(e) => onBandInput(i, e)}
        />
        <span class="hz mono">{fmtHz(hz)}</span>
      </div>
    {/each}
  </div>
</section>

<style>
  .eq {
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
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

  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-family: var(--ac-font-mono);
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .presets button {
    text-transform: capitalize;
    font-size: 0.85rem;
    padding: 0.35rem 0.75rem;
  }

  .bands {
    display: grid;
    grid-template-columns: repeat(10, 1fr);
    gap: 0.6rem;
  }

  .band {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 0.25rem;
    border-radius: var(--ac-radius);
    background: var(--ac-bg-elevated);
    border: 1px solid var(--ac-border);
  }

  .db,
  .hz {
    font-size: 0.7rem;
    color: var(--ac-text-muted);
    letter-spacing: 0.04em;
  }

  input[type='range'] {
    -webkit-appearance: slider-vertical;
    appearance: slider-vertical;
    writing-mode: vertical-lr;
    direction: rtl;
    width: 1.2rem;
    height: 160px;
    accent-color: var(--ac-accent);
  }

  @media (max-width: 720px) {
    .bands {
      grid-template-columns: repeat(5, 1fr);
    }
  }
</style>

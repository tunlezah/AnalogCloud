<script lang="ts">
  import { onMount } from 'svelte';
  import Equalizer from '$lib/eq/Equalizer.svelte';
  import { api } from '$lib/api/client';
  import type { EqualizerSettings } from '$lib/api/types';

  let eq: EqualizerSettings = {
    enabled: true,
    preset: 'flat',
    bands_db: new Array(10).fill(0)
  };

  onMount(async () => {
    try {
      const s = await api.settings();
      eq = s.equalizer;
    } catch {
      // backend offline — keep defaults
    }
  });

  async function save() {
    try {
      await api.patchSettings({ equalizer: eq });
    } catch (err) {
      console.warn('save failed', err);
    }
  }
</script>

<Equalizer bind:value={eq} />

<div class="actions">
  <button data-variant="accent" on:click={save}>Save Preset</button>
</div>

<style>
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>

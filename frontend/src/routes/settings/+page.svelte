<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';
  import type { Settings } from '$lib/api/types';

  let settings: Settings | null = null;
  let saving = false;
  let savedAt: string | null = null;

  onMount(async () => {
    try {
      settings = await api.settings();
    } catch {
      // backend offline — show form anyway with defaults
      settings = {
        audio_mode: 'reference',
        equalizer: { enabled: true, preset: 'flat', bands_db: new Array(10).fill(0) },
        enrichment: { mode: 'delayed', delayed_wait_seconds: 30 },
        hass: { enabled: false, mqtt_host: null, mqtt_port: null, mqtt_username: null, mqtt_topic_prefix: null },
        theme: 'analog-core'
      };
    }
  });

  async function save() {
    if (!settings) return;
    saving = true;
    try {
      settings = await api.patchSettings(settings);
      savedAt = new Date().toLocaleTimeString();
    } catch (err) {
      console.warn('save failed', err);
    } finally {
      saving = false;
    }
  }
</script>

{#if settings}
  <section class="panel form">
    <h2>Audio mode</h2>
    <div class="row">
      <label>
        <input type="radio" bind:group={settings.audio_mode} value="reference" />
        <span><strong>Reference</strong> — prioritize fidelity</span>
      </label>
      <label>
        <input type="radio" bind:group={settings.audio_mode} value="compatible" />
        <span><strong>Compatible</strong> — prioritize reliability</span>
      </label>
    </div>

    <h2>Online enrichment</h2>
    <div class="row">
      {#each ['off', 'immediate', 'delayed', 'manual'] as mode}
        <label>
          <input type="radio" bind:group={settings.enrichment.mode} value={mode} />
          <span style="text-transform: capitalize">{mode}</span>
        </label>
      {/each}
    </div>
    {#if settings.enrichment.mode === 'delayed'}
      <label class="inline">
        <span class="muted small">Wait seconds</span>
        <input
          type="number"
          min="5"
          max="600"
          bind:value={settings.enrichment.delayed_wait_seconds}
        />
      </label>
    {/if}

    <h2>Home Assistant</h2>
    <label class="inline">
      <input type="checkbox" bind:checked={settings.hass.enabled} />
      <span>Publish state to HASS via MQTT</span>
    </label>

    <div class="actions">
      <span class="muted small">
        {savedAt ? `saved ${savedAt}` : 'not saved'}
      </span>
      <button data-variant="accent" disabled={saving} on:click={save}>
        {saving ? 'Saving…' : 'Save'}
      </button>
    </div>
  </section>
{/if}

<style>
  .form {
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  h2 {
    font-size: 0.95rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ac-text-muted);
    margin-top: 0.5rem;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  label {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.7rem;
    border-radius: var(--ac-radius);
    background: var(--ac-bg-elevated);
    border: 1px solid var(--ac-border);
    cursor: pointer;
  }
  label.inline { background: transparent; border: none; padding: 0.25rem 0; }
  .small {
    font-size: 0.7rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  input[type='number'] {
    width: 6rem;
    background: var(--ac-bg-elevated);
    color: var(--ac-text);
    border: 1px solid var(--ac-border);
    border-radius: var(--ac-radius);
    padding: 0.35rem 0.55rem;
    font: inherit;
  }
  .actions {
    margin-top: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
</style>

<script lang="ts">
  import { activeSession, inputs, outputs } from '$lib/stores/session';
  import { themeId } from '$lib/themes/store';
  import { themeList } from '$lib/themes/themes';

  $: input = $inputs.find((i) => $activeSession && i.id === $activeSession.input_id);
  $: output = $outputs.find(
    (o) => $activeSession && o.kind === $activeSession.output_kind && o.name === $activeSession.output_device
  );
</script>

<header class="topbar panel">
  <div class="brand">
    <img src="/icon.png" alt="Analog Cloud" width="32" height="32" />
    <div class="title">
      <span class="display">Analog Cloud</span>
      <span class="muted small">physical audio · rebroadcast beautifully</span>
    </div>
  </div>

  <div class="slot">
    <span class="muted small">Input</span>
    <strong>{input?.name ?? '—'}</strong>
  </div>

  <div class="arrow" aria-hidden="true">→</div>

  <div class="slot">
    <span class="muted small">Output</span>
    <strong>{output?.name ?? '—'}</strong>
  </div>

  <div class="spacer" />

  <label class="theme-switch">
    <span class="muted small">Theme</span>
    <select bind:value={$themeId}>
      {#each themeList as t}
        <option value={t.id}>{t.name}</option>
      {/each}
    </select>
  </label>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.6rem 1rem;
    margin: 0.75rem 0.75rem 0 0.75rem;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .brand img {
    border-radius: 8px;
  }

  .title {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
  }

  .display {
    font-family: var(--ac-font-display);
    font-weight: var(--ac-weight-display);
    font-size: 1.05rem;
    letter-spacing: 0.04em;
  }

  .small {
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .slot {
    display: flex;
    flex-direction: column;
    line-height: 1.2;
    min-width: 7rem;
  }

  .arrow {
    color: var(--ac-accent);
    font-size: 1.1rem;
  }

  .spacer {
    flex: 1;
  }

  .theme-switch {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  select {
    background: var(--ac-bg-elevated);
    color: var(--ac-text);
    border: 1px solid var(--ac-border);
    border-radius: var(--ac-radius);
    padding: 0.35rem 0.55rem;
    font: inherit;
  }
</style>

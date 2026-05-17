<script lang="ts">
  import { outputs, activeSession, inputs } from '$lib/stores/session';
  import { api } from '$lib/api/client';
  import OutputCard from '$lib/components/OutputCard.svelte';
  import type { OutputDescriptor } from '$lib/api/types';

  $: activeKey =
    $activeSession && `${$activeSession.output_kind}::${$activeSession.output_device}`;

  function isActive(o: OutputDescriptor) {
    return activeKey === `${o.kind}::${o.name}`;
  }

  async function takeOver(o: OutputDescriptor) {
    const sourceId = $activeSession?.input_id ?? $inputs[0]?.id;
    if (!sourceId) return;
    try {
      await api.startSession({
        input_id: sourceId,
        output_kind: o.kind,
        output_device: o.name
      });
    } catch (err) {
      console.warn('takeover failed', err);
    }
  }
</script>

<header><h2>Outputs</h2></header>

<div class="grid">
  {#each $outputs as output (output.id)}
    <OutputCard {output} active={isActive(output)} on:click={() => takeOver(output)} />
  {/each}
</div>

<style>
  header h2 {
    font-size: 1rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ac-text-muted);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.6rem;
  }
</style>

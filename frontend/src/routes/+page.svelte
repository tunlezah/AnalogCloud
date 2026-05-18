<script lang="ts">
  import { inputs, outputs, activeSession } from '$lib/stores/session';
  import { api } from '$lib/api/client';
  import InputCard from '$lib/components/InputCard.svelte';
  import OutputCard from '$lib/components/OutputCard.svelte';
  import SessionPanel from '$lib/session_ui/SessionPanel.svelte';
  import type { InputDescriptor, OutputDescriptor } from '$lib/api/types';

  let selectedInputId: string | null = null;

  $: if (!selectedInputId && $inputs.length) selectedInputId = $inputs[0].id;

  async function selectInput(i: InputDescriptor) {
    selectedInputId = i.id;
  }

  async function takeOver(o: OutputDescriptor) {
    if (!selectedInputId) return;
    try {
      await api.startSession({
        input_id: selectedInputId,
        output_kind: o.kind,
        output_device: o.name
      });
    } catch (err) {
      console.warn('start failed', err);
    }
  }

  $: activeOutputKey =
    $activeSession && `${$activeSession.output_kind}::${$activeSession.output_device}`;
  function isActive(o: OutputDescriptor) {
    return activeOutputKey === `${o.kind}::${o.name}`;
  }
</script>

<SessionPanel />

<section class="row">
  <div class="col">
    <header><h2>Inputs</h2></header>
    {#if $inputs.length === 0}
      <p class="muted">No inputs discovered yet. Pair a Bluetooth source or plug in line-in.</p>
    {/if}
    <div class="grid">
      {#each $inputs as input (input.id)}
        <InputCard {input} selected={input.id === selectedInputId} on:click={() => selectInput(input)} />
      {/each}
    </div>
  </div>

  <div class="col">
    <header><h2>Outputs</h2></header>
    {#if $outputs.length === 0}
      <p class="muted">No outputs available.</p>
    {/if}
    <div class="grid">
      {#each $outputs as output (output.id)}
        <OutputCard {output} active={isActive(output)} on:click={() => takeOver(output)} />
      {/each}
    </div>
  </div>
</section>

<style>
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .col {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  header h2 {
    font-size: 1rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ac-text-muted);
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.5rem;
  }

  @media (max-width: 1100px) {
    .row {
      grid-template-columns: 1fr;
    }
  }
</style>

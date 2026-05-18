<script lang="ts">
  import SessionPanel from '$lib/session_ui/SessionPanel.svelte';
  import { activeSession } from '$lib/stores/session';
</script>

<SessionPanel />

<section class="panel meta">
  <h2>Session record</h2>
  {#if $activeSession}
    <pre class="mono">{JSON.stringify($activeSession, null, 2)}</pre>
  {:else}
    <p class="muted">No active session.</p>
  {/if}
</section>

<style>
  .meta {
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  h2 {
    font-size: 1rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ac-text-muted);
  }
  pre {
    background: var(--ac-bg-elevated);
    border: 1px solid var(--ac-border);
    border-radius: var(--ac-radius);
    padding: 0.85rem 1rem;
    font-size: 0.85rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>

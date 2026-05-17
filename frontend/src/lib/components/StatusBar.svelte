<script lang="ts">
  import { activeSession, backendOnline } from '$lib/stores/session';

  $: stats = $activeSession?.stats;
</script>

<footer class="status panel">
  <span class="badge" data-tone={$backendOnline ? 'live' : undefined}>
    {$backendOnline ? 'Backend OK' : 'Backend offline'}
  </span>
  <span class="cell">
    <span class="muted small">latency</span>
    <span class="mono">{stats ? `${stats.latency_ms} ms` : '—'}</span>
  </span>
  <span class="cell">
    <span class="muted small">codec</span>
    <span class="mono">{stats?.codec ?? '—'}</span>
  </span>
  <span class="cell">
    <span class="muted small">bitrate</span>
    <span class="mono">{stats?.bitrate_kbps ? `${stats.bitrate_kbps} kbps` : '—'}</span>
  </span>
  <span class="cell">
    <span class="muted small">buffer</span>
    <span class="mono">{stats ? `${stats.buffer_ms} ms` : '—'}</span>
  </span>
  <span class="spacer" />
  <span class="muted small">analog · cloud</span>
</footer>

<style>
  .status {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    margin: 0 0.75rem 0.75rem 0.75rem;
    font-size: 0.85rem;
  }

  .cell {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
  }

  .small {
    font-size: 0.65rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .spacer { flex: 1; }
</style>

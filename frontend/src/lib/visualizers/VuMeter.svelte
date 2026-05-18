<script lang="ts">
  import { levels } from '$lib/stores/session';

  $: leftPct = clamp($levels.l);
  $: rightPct = clamp($levels.r);

  function clamp(dbfs: number): number {
    const min = -60;
    if (dbfs <= min) return 0;
    if (dbfs >= 0) return 100;
    return ((dbfs - min) / -min) * 100;
  }
</script>

<div class="vu">
  <div class="channel">
    <span class="muted small">L</span>
    <div class="meter"><span style="--w: {leftPct}%" /></div>
  </div>
  <div class="channel">
    <span class="muted small">R</span>
    <div class="meter"><span style="--w: {rightPct}%" /></div>
  </div>
</div>

<style>
  .vu {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.6rem 0.8rem;
    border: 1px solid var(--ac-border);
    border-radius: var(--ac-radius);
    background: var(--ac-bg-elevated);
  }

  .channel {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .small {
    font-size: 0.65rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    width: 1ch;
  }

  .meter {
    flex: 1;
    height: 8px;
    border-radius: 4px;
    background: var(--ac-bg);
    overflow: hidden;
    border: 1px solid var(--ac-border);
  }

  .meter span {
    display: block;
    height: 100%;
    width: var(--w, 0%);
    background: linear-gradient(
      90deg,
      var(--ac-indicator) 0%,
      var(--ac-accent) 70%,
      var(--ac-danger) 100%
    );
    transition: width 60ms linear;
  }
</style>

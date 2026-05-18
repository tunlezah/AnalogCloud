<script lang="ts">
  import { themeId, setTheme } from '$lib/themes/store';
  import { themeList } from '$lib/themes/themes';
</script>

<section class="grid">
  {#each themeList as t}
    <button
      class="card theme-card"
      data-active={$themeId === t.id}
      on:click={() => setTheme(t.id)}
    >
      <div class="swatch" style:--p={t.color.accent} style:--s={t.color.indicator}>
        <span class="bg" style:background={t.color.bg} />
        <span class="dot a" />
        <span class="dot b" />
      </div>
      <strong>{t.name}</strong>
      <span class="muted small">{t.blurb}</span>
    </button>
  {/each}
</section>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 0.75rem;
  }

  .theme-card {
    text-align: left;
    cursor: pointer;
  }

  .swatch {
    position: relative;
    aspect-ratio: 16 / 9;
    border-radius: var(--ac-radius);
    overflow: hidden;
    border: 1px solid var(--ac-border);
  }

  .swatch .bg {
    position: absolute;
    inset: 0;
  }

  .swatch .dot {
    position: absolute;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
  }

  .swatch .dot.a {
    background: var(--p);
    left: 18%;
    top: 50%;
    transform: translateY(-50%);
  }

  .swatch .dot.b {
    background: var(--s);
    right: 18%;
    top: 50%;
    transform: translateY(-50%);
    opacity: 0.85;
  }

  .small {
    font-size: 0.7rem;
    letter-spacing: 0.03em;
    text-transform: none;
  }
</style>

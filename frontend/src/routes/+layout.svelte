<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import LeftPanel from '$lib/components/LeftPanel.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import { theme } from '$lib/themes/store';
  import { applyTheme } from '$lib/themes/store';
  import { bootstrap } from '$lib/stores/session';

  onMount(() => {
    applyTheme($theme);
    bootstrap();
  });
</script>

<div class="shell">
  <TopBar />
  <main class="body">
    <LeftPanel />
    <section class="content">
      <slot />
    </section>
  </main>
  <StatusBar />
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
    min-height: 0;
  }

  .body {
    display: grid;
    grid-template-columns: 280px 1fr;
    min-height: 0;
    gap: 0.75rem;
    padding: 0.75rem;
  }

  .content {
    overflow: auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  @media (max-width: 900px) {
    .body {
      grid-template-columns: 1fr;
    }
  }
</style>

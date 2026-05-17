<script lang="ts">
  import { page } from '$app/stores';

  const nav = [
    { href: '/', label: 'Inputs' },
    { href: '/outputs', label: 'Outputs' },
    { href: '/sessions', label: 'Sessions' },
    { href: '/eq', label: 'Equalizer' },
    { href: '/themes', label: 'Themes' },
    { href: '/settings', label: 'Settings' }
  ];
</script>

<aside class="panel left">
  <nav>
    {#each nav as item}
      <a href={item.href} data-active={$page.url.pathname === item.href}>
        <span class="dot" />
        {item.label}
      </a>
    {/each}
  </nav>

  <footer class="muted small">
    <div>Analog Cloud v0.1.0</div>
    <div>local-first · offline-capable</div>
  </footer>
</aside>

<style>
  .left {
    padding: 0.85rem;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  a {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.55rem 0.7rem;
    border-radius: var(--ac-radius);
    color: var(--ac-text-muted);
    border: 1px solid transparent;
    transition: all var(--ac-motion-fast) var(--ac-easing);
  }

  a:hover {
    color: var(--ac-text);
    background: var(--ac-bg-elevated);
  }

  a[data-active='true'] {
    color: var(--ac-text);
    background: var(--ac-bg-elevated);
    border-color: var(--ac-border);
  }

  a[data-active='true'] .dot {
    background: var(--ac-accent);
    box-shadow: 0 0 8px var(--ac-accent);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ac-border);
    transition: all var(--ac-motion-fast) var(--ac-easing);
  }

  footer {
    margin-top: auto;
    padding-top: 1rem;
    border-top: 1px solid var(--ac-border);
    line-height: 1.5;
  }

  .small {
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
</style>

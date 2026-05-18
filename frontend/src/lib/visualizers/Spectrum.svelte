<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  export let bars = 64;
  export let height = 220;

  let canvas: HTMLCanvasElement;
  let raf = 0;
  let values: number[] = new Array(bars).fill(0);

  onMount(() => {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const resize = () => {
      const w = canvas.parentElement?.clientWidth ?? 600;
      canvas.width = w * dpr;
      canvas.height = height * dpr;
      canvas.style.width = `${w}px`;
      canvas.style.height = `${height}px`;
      ctx.scale(dpr, dpr);
    };
    resize();
    window.addEventListener('resize', resize);

    const tick = () => {
      values = values.map((v, i) => {
        const target =
          0.5 + 0.5 * Math.sin(performance.now() / (220 + i * 6) + i * 0.4);
        return v + (target - v) * 0.12;
      });
      draw(ctx);
      raf = requestAnimationFrame(tick);
    };
    tick();

    return () => {
      window.removeEventListener('resize', resize);
    };
  });

  onDestroy(() => cancelAnimationFrame(raf));

  function draw(ctx: CanvasRenderingContext2D) {
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    ctx.clearRect(0, 0, w, h);

    const gap = 3;
    const bw = (w - gap * (bars - 1)) / bars;
    const primary = cssVar('--ac-viz-primary', '#F0A23B');
    const secondary = cssVar('--ac-viz-secondary', '#5BE49B');

    for (let i = 0; i < bars; i++) {
      const v = values[i];
      const bh = Math.max(2, v * h);
      const x = i * (bw + gap);
      const y = h - bh;
      const grad = ctx.createLinearGradient(0, y, 0, h);
      grad.addColorStop(0, primary);
      grad.addColorStop(1, secondary);
      ctx.fillStyle = grad;
      ctx.fillRect(x, y, bw, bh);
    }
  }

  function cssVar(name: string, fallback: string): string {
    if (typeof window === 'undefined') return fallback;
    return (
      getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback
    );
  }
</script>

<canvas bind:this={canvas} aria-hidden="true" />

<style>
  canvas {
    display: block;
    width: 100%;
    border-radius: var(--ac-radius);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.02), transparent 30%),
      var(--ac-bg-elevated);
    border: 1px solid var(--ac-border);
  }
</style>

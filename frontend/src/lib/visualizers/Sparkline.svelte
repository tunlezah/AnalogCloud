<script lang="ts">
  export let value: number;
  export let history = 64;

  let buf: number[] = new Array(history).fill(-60);

  $: pushValue(value);

  function pushValue(v: number) {
    buf = buf.slice(1).concat([v]);
  }

  function pathFor(values: number[]): string {
    const w = 100;
    const h = 24;
    const min = -60;
    const max = 0;
    return values
      .map((v, i) => {
        const x = (i / (values.length - 1)) * w;
        const clamped = Math.min(max, Math.max(min, v));
        const y = h - ((clamped - min) / (max - min)) * h;
        return `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(' ');
  }
</script>

<svg viewBox="0 0 100 24" preserveAspectRatio="none" class="sparkline">
  <path d={pathFor(buf)} fill="none" stroke="var(--ac-viz-primary)" stroke-width="1.2" />
</svg>

<style>
  .sparkline {
    width: 100%;
    height: 24px;
    opacity: 0.8;
  }
</style>

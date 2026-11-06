<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { settings } from '../lib/state.svelte';

  // Светящаяся «дышащая» линия вокруг обложки (приём из Doppo).
  // Рисуется на canvas ~30 раз в секунду, цвет — из обложки (--cover-glow).
  let { target, playing = true }: { target: HTMLElement | undefined; playing?: boolean } = $props();

  let canvas: HTMLCanvasElement;
  let raf = 0;
  let last = 0;
  let t0 = 0;
  let color = '#e6efff';
  let colorAt = 0;
  const reduce = typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;

  function resize() {
    if (!canvas) return;
    const r = canvas.getBoundingClientRect();
    const d = Math.min(2, devicePixelRatio || 1);
    canvas.width = Math.max(1, Math.round(r.width * d));
    canvas.height = Math.max(1, Math.round(r.height * d));
    canvas.getContext('2d')!.setTransform(d, 0, 0, d, 0, 0);
  }

  function draw(t: number) {
    const ctx = canvas?.getContext('2d');
    if (!ctx || !target) return;
    const box = canvas.getBoundingClientRect();
    ctx.clearRect(0, 0, box.width, box.height);
    if (!settings.fx) return;
    if (t - colorAt > 400) {
      colorAt = t;
      color = getComputedStyle(canvas).getPropertyValue('--cover-glow').trim() || color;
    }
    const c = target.getBoundingClientRect();
    const cx = c.left - box.left + c.width / 2;
    const cy = c.top - box.top + c.height / 2;
    const base = c.width * 0.66;
    for (const [lw, blur, alpha, k] of [
      [6, 48, 0.12, 1.06],
      [2, 18, 0.85, 1],
    ] as const) {
      ctx.beginPath();
      for (let i = 0; i <= 120; i++) {
        const a = (i / 120) * Math.PI * 2;
        const n =
          Math.sin(a * 3 + t * 0.0006) * 0.12 + Math.sin(a * 5 - t * 0.0004) * 0.07 + Math.sin(a * 2 + t * 0.0003) * 0.1;
        const rr = base * k * (1 + n);
        const x = cx + Math.cos(a) * rr * 1.25;
        const y = cy + Math.sin(a) * rr * 0.95;
        if (i) ctx.lineTo(x, y);
        else ctx.moveTo(x, y);
      }
      ctx.closePath();
      ctx.strokeStyle = color;
      ctx.globalAlpha = alpha;
      ctx.lineWidth = lw;
      ctx.shadowColor = color;
      ctx.shadowBlur = blur;
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
    ctx.shadowBlur = 0;
  }

  function loop(t: number) {
    raf = requestAnimationFrame(loop);
    if (document.hidden) return;
    // на паузе аура почти замирает
    const speed = playing ? 1 : 0.15;
    if (t - last < 33) return;
    t0 += (t - last) * speed;
    last = t;
    draw(t0);
  }

  onMount(() => {
    resize();
    const ro = new ResizeObserver(() => {
      resize();
      draw(t0);
    });
    ro.observe(canvas);
    if (reduce) draw(0);
    else raf = requestAnimationFrame(loop);
    return () => ro.disconnect();
  });
  onDestroy(() => cancelAnimationFrame(raf));
  $effect(() => {
    void settings.fx;
    draw(t0);
  });
</script>

<canvas class="aura" bind:this={canvas} aria-hidden="true"></canvas>

<style>
  .aura {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 0;
  }
</style>

<script lang="ts">
  import { api } from '../lib/api';
  import Icon from './Icon.svelte';

  let {
    src = '',
    size = 48,
    radius = 8,
    alt = '',
  }: { src?: string; size?: number | string; radius?: number; alt?: string } = $props();

  // 0 — через Rust-прокси, 1 — напрямую, 2 — заглушка
  let attempt = $state(0);
  let lastSrc = '';
  $effect(() => {
    if (src !== lastSrc) {
      lastSrc = src;
      attempt = 0;
    }
  });

  const url = $derived(attempt === 0 ? api.img(src) : src);
  const dim = $derived(typeof size === 'number' ? `${size}px` : size);
  // У превью YouTube (hqdefault/sddefault) чёрные полосы сверху и снизу — обрезаем их.
  const letterbox = $derived(/i\.ytimg\.com\/vi\/[^/]+\/(hq|sd)default/.test(src));
</script>

<div class="cover" style="width:{dim};height:{dim};border-radius:{radius}px">
  {#if src && attempt < 2}
    <img
      src={url}
      {alt}
      class:letterbox
      loading="lazy"
      referrerpolicy="no-referrer"
      onerror={() => (attempt += 1)}
    />
  {:else}
    <div class="ph"><Icon name="note" size={typeof size === 'number' ? Math.max(14, size * 0.36) : 40} /></div>
  {/if}
</div>

<style>
  .cover {
    position: relative;
    flex: none;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.06);
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  img.letterbox {
    transform: scale(1.34);
  }
  .ph {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    color: rgba(255, 255, 255, 0.35);
    background: linear-gradient(135deg, #2a2a33, #1b1b22);
  }
</style>

<script lang="ts">
  let {
    src = '',
    size = 48,
    radius = 12,
    alt = '',
  }: { src?: string; size?: number | string; radius?: number; alt?: string } = $props();
  let failed = $state(false);
  const dim = $derived(typeof size === 'number' ? `${size}px` : size);
</script>

<div class="cover" style="width:{dim};height:{dim};border-radius:{radius}px">
  {#if src && !failed}
    <img {src} {alt} loading="lazy" referrerpolicy="no-referrer" onerror={() => (failed = true)} />
  {:else}
    <div class="ph">
      <svg viewBox="0 0 24 24" width="40%" height="40%" fill="none" stroke="currentColor" stroke-width="1.6"
        ><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg
      >
    </div>
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
  .ph {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    color: rgba(255, 255, 255, 0.4);
    background: linear-gradient(135deg, #2a2a33, #1b1b22);
  }
</style>

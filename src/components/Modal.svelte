<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    onclose,
    width = 460,
    children,
  }: { onclose?: () => void; width?: number; children: Snippet } = $props();

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && onclose) onclose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="overlay" role="presentation" onmousedown={(e) => e.target === e.currentTarget && onclose?.()}>
  <div class="modal glass" style="width:min({width}px, calc(100vw - 32px))" role="dialog" aria-modal="true">
    {#if onclose}
      <button class="icon-btn x" onclick={onclose} aria-label="Закрыть"><Icon name="x" size={18} /></button>
    {/if}
    {@render children()}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
    animation: fade 0.2s ease both;
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  .modal {
    position: relative;
    border-radius: var(--r-xl);
    padding: 24px;
    max-height: calc(100vh - 80px);
    overflow-y: auto;
    background: rgba(30, 30, 37, 0.94);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5);
    animation: pop 0.18s ease both;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
  }
  .x {
    position: absolute;
    top: 14px;
    right: 14px;
  }
</style>

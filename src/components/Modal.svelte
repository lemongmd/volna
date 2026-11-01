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
    background: rgba(5, 3, 12, 0.55);
    backdrop-filter: blur(6px);
    animation: fade 0.2s ease both;
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  .modal {
    position: relative;
    border-radius: 30px;
    padding: 30px;
    max-height: calc(100vh - 80px);
    overflow-y: auto;
    background: linear-gradient(150deg, rgba(48, 32, 84, 0.82), rgba(18, 12, 34, 0.86));
    animation: pop 0.3s cubic-bezier(0.2, 0.9, 0.3, 1.2) both;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.94) translateY(10px);
    }
  }
  .x {
    position: absolute;
    top: 16px;
    right: 16px;
  }
</style>

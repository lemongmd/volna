<script lang="ts">
  import { toasts } from '../lib/state.svelte';
</script>

<div class="toasts" aria-live="polite">
  {#each toasts as t (t.id)}
    <div class="toast glass" class:error={t.kind === 'error'}>{t.text}</div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    left: calc(50% + 32px);
    bottom: 92px;
    transform: translateX(-50%);
    z-index: 90;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    pointer-events: none;
  }
  .toast {
    padding: 9px 14px;
    border-radius: 10px;
    font-size: 13px;
    max-width: 520px;
    text-align: center;
    background: rgba(36, 36, 44, 0.92);
    animation: fade-in 0.2s ease both;
  }
  .toast.error {
    background: rgba(70, 24, 34, 0.94);
    border-color: rgba(255, 107, 129, 0.3);
  }
</style>

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
    left: 50%;
    bottom: 120px;
    transform: translateX(-50%);
    z-index: 90;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    pointer-events: none;
  }
  .toast {
    padding: 10px 18px;
    border-radius: 99px;
    font-weight: 500;
    max-width: 560px;
    text-align: center;
    background: linear-gradient(135deg, rgba(60, 40, 100, 0.85), rgba(25, 16, 46, 0.85));
    animation: toast-in 0.3s cubic-bezier(0.2, 0.9, 0.3, 1.2) both;
  }
  .toast.error {
    background: linear-gradient(135deg, rgba(120, 30, 60, 0.9), rgba(60, 16, 34, 0.9));
    border-color: rgba(251, 113, 133, 0.4);
  }
  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.95);
    }
  }
</style>

<script lang="ts">
  import { current, settings } from '../lib/state.svelte';

  // Фон окрашивается обложкой текущего трека: она сильно размыта и затемнена.
  const thumb = $derived(settings.fx ? (current()?.thumb ?? '') : '');
</script>

<div class="backdrop" aria-hidden="true">
  {#key thumb}
    {#if thumb}
      <img class="cover" src={thumb} alt="" referrerpolicy="no-referrer" />
    {/if}
  {/key}
  <div class="shade"></div>
</div>

<style>
  .backdrop {
    position: absolute;
    inset: 0;
    z-index: 0;
    overflow: hidden;
    pointer-events: none;
    background: radial-gradient(1000px 600px at 30% -10%, #1f2130 0%, transparent 70%), var(--bg);
  }
  .cover {
    position: absolute;
    inset: -15%;
    width: 130%;
    height: 130%;
    object-fit: cover;
    filter: blur(100px) saturate(1.15) brightness(0.8);
    opacity: 0.6;
    transform: translateZ(0);
    animation: cover-in 0.9s ease both;
  }
  @keyframes cover-in {
    from {
      opacity: 0;
    }
  }
  .shade {
    position: absolute;
    inset: 0;
    background: linear-gradient(180deg, rgba(14, 14, 18, 0.35) 0%, rgba(14, 14, 18, 0.7) 55%, rgba(14, 14, 18, 0.9) 100%);
  }
</style>

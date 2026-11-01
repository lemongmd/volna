<script lang="ts">
  import { current } from '../lib/state.svelte';

  // Обложка текущего трека, сильно размытая, — фон «окрашивается» под музыку.
  const thumb = $derived(current()?.thumb ?? '');
</script>

<div class="backdrop" aria-hidden="true">
  <div class="base"></div>
  {#key thumb}
    {#if thumb}
      <img class="cover" src={thumb} alt="" referrerpolicy="no-referrer" />
    {/if}
  {/key}
  <div class="blob b1"></div>
  <div class="blob b2"></div>
  <div class="blob b3"></div>
  <div class="grid"></div>
  <div class="vignette"></div>
</div>

<style>
  .backdrop {
    position: absolute;
    inset: 0;
    z-index: 0;
    overflow: hidden;
    pointer-events: none;
  }
  .base {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(1200px 700px at 15% -10%, #2a1650 0%, transparent 60%),
      radial-gradient(900px 600px at 110% 110%, #0f2c4a 0%, transparent 60%),
      linear-gradient(180deg, #0c0817, #07060d);
  }
  .cover {
    position: absolute;
    inset: -20%;
    width: 140%;
    height: 140%;
    object-fit: cover;
    filter: blur(90px) saturate(1.7) brightness(0.75);
    opacity: 0.55;
    animation: cover-in 1.2s ease both;
  }
  @keyframes cover-in {
    from {
      opacity: 0;
    }
  }
  .blob {
    position: absolute;
    border-radius: 50%;
    filter: blur(80px);
    opacity: 0.45;
    will-change: transform;
  }
  .b1 {
    width: 520px;
    height: 520px;
    left: -120px;
    top: 10%;
    background: #7c3aed;
  }
  .b2 {
    width: 460px;
    height: 460px;
    right: -100px;
    top: -120px;
    background: #c026d3;
    opacity: 0.28;
  }
  .b3 {
    width: 600px;
    height: 600px;
    right: 10%;
    bottom: -320px;
    background: #0ea5e9;
    opacity: 0.25;
  }
  :global(.fx) .b1 {
    animation: drift1 28s ease-in-out infinite alternate;
  }
  :global(.fx) .b2 {
    animation: drift2 34s ease-in-out infinite alternate;
  }
  :global(.fx) .b3 {
    animation: drift3 40s ease-in-out infinite alternate;
  }
  @keyframes drift1 {
    to {
      transform: translate(180px, 120px) scale(1.15);
    }
  }
  @keyframes drift2 {
    to {
      transform: translate(-160px, 140px) scale(0.9);
    }
  }
  @keyframes drift3 {
    to {
      transform: translate(-200px, -80px) scale(1.1);
    }
  }
  .grid {
    position: absolute;
    inset: 0;
    background-image:
      linear-gradient(rgba(255, 255, 255, 0.035) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.035) 1px, transparent 1px);
    background-size: 56px 56px;
    mask-image: radial-gradient(ellipse at 50% 30%, #000 10%, transparent 75%);
  }
  .vignette {
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse at center, transparent 40%, rgba(5, 3, 10, 0.75) 100%);
  }
</style>

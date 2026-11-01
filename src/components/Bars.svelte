<script lang="ts">
  // Лёгкий «эквалайзер» на CSS: не нагружает процессор, в отличие от анализатора.
  let { playing = true, size = 14, count = 4 }: { playing?: boolean; size?: number; count?: number } = $props();
</script>

<span class="bars" class:paused={!playing} style="height:{size}px;gap:{Math.max(2, size / 7)}px" aria-hidden="true">
  {#each Array(count) as _, i}
    <i style="animation-delay:{-i * 0.23}s;animation-duration:{0.9 + (i % 3) * 0.17}s"></i>
  {/each}
</span>

<style>
  .bars {
    display: inline-flex;
    align-items: flex-end;
    flex: none;
  }
  i {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: linear-gradient(180deg, #e9d5ff, var(--accent));
    transform-origin: bottom;
    animation: eq 1s ease-in-out infinite alternate;
  }
  .paused i {
    animation-play-state: paused;
  }
  @keyframes eq {
    0% {
      transform: scaleY(0.2);
    }
    50% {
      transform: scaleY(1);
    }
    100% {
      transform: scaleY(0.45);
    }
  }
</style>

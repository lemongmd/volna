<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Bars from './Bars.svelte';
  import { current, ps } from '../lib/state.svelte';
  import { addToQueue, playNow } from '../lib/sync';
  import type { Track } from '../lib/types';
  import { hoverEnd, hoverStart } from '../lib/prefetch';

  let { track }: { track: Track } = $props();
  const isCur = $derived(current()?.id === track.id);
</script>

<div class="card" class:cur={isCur} onmouseenter={() => hoverStart(track)} onmouseleave={hoverEnd} role="group">
  <div class="art">
    <Cover src={track.thumb} size="100%" radius={12} />
    <div class="hover"></div>
    <button class="play" onclick={() => playNow(track)} aria-label="Играть «{track.title}»">
      {#if isCur && ps.playing}<Bars size={14} />{:else}<Icon name="play" size={16} />{/if}
    </button>
    <button class="add" onclick={() => addToQueue([track])} title="В очередь" aria-label="В очередь">
      <Icon name="plus" size={15} />
    </button>
  </div>
  <div class="title">
    {#if track.explicit}<span class="badge-e">E</span>{/if}
    <span class="ellipsis" title={track.title}>{track.title}</span>
  </div>
  <div class="artist ellipsis">{track.artist}</div>
</div>

<style>
  .card {
    min-width: 0;
  }
  .art {
    position: relative;
    aspect-ratio: 1;
    border-radius: 12px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.28);
  }
  .hover {
    position: absolute;
    inset: 0;
    border-radius: 12px;
    background: rgba(0, 0, 0, 0.25);
    opacity: 0;
    transition: opacity 0.15s;
    pointer-events: none;
  }
  .card:hover .hover {
    opacity: 1;
  }
  .play {
    position: absolute;
    left: 12px;
    bottom: 12px;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--light);
    color: var(--light-ink);
    opacity: 0;
    transition: opacity 0.15s, transform 0.1s;
  }
  .play :global(svg) {
    margin-left: 2px;
  }
  .play :global(.bars i) {
    background: var(--light-ink);
  }
  .play:hover {
    transform: scale(1.06);
  }
  .cur .play,
  .card:hover .play {
    opacity: 1;
  }
  .add {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 30px;
    height: 30px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    background: rgba(20, 20, 26, 0.6);
    backdrop-filter: blur(10px);
    opacity: 0;
    transition: opacity 0.15s;
  }
  .card:hover .add {
    opacity: 1;
  }
  .add:hover {
    background: rgba(20, 20, 26, 0.85);
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: var(--s3);
    font-weight: 550;
    min-width: 0;
  }
  .artist {
    color: var(--muted);
    font-size: 13px;
    margin-top: 3px;
  }
</style>

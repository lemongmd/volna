<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Bars from './Bars.svelte';
  import { current, ps } from '../lib/state.svelte';
  import { addToQueue, playNow } from '../lib/sync';
  import type { Track } from '../lib/types';

  let { track }: { track: Track } = $props();
  const isCur = $derived(current()?.id === track.id);
</script>

<div class="card" class:cur={isCur}>
  <div class="art">
    <Cover src={track.thumb} size="100%" radius={18} />
    <div class="shine"></div>
    <button class="play" onclick={() => playNow(track)} aria-label="Играть «{track.title}»">
      {#if isCur && ps.playing}<Bars size={16} />{:else}<Icon name="play" size={20} />{/if}
    </button>
    <button class="add" onclick={() => addToQueue([track])} title="В очередь" aria-label="В очередь">
      <Icon name="plus" size={16} />
    </button>
  </div>
  <div class="title ellipsis" title={track.title}>
    {#if track.explicit}<span class="badge-e">E</span>{/if}
    {track.title}
  </div>
  <div class="artist ellipsis">{track.artist}</div>
</div>

<style>
  .card {
    min-width: 0;
    padding: 10px;
    margin: -10px;
    border-radius: 24px;
    transition: background 0.2s;
  }
  .card:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .art {
    position: relative;
    aspect-ratio: 1;
    border-radius: 18px;
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.45);
    transition: transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .card:hover .art {
    transform: translateY(-3px);
  }
  .shine {
    position: absolute;
    inset: 0;
    border-radius: 18px;
    background: linear-gradient(160deg, rgba(255, 255, 255, 0.22), transparent 40%);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12);
    pointer-events: none;
  }
  .play {
    position: absolute;
    right: 10px;
    bottom: 10px;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.92);
    color: #140c28;
    box-shadow: 0 8px 22px rgba(0, 0, 0, 0.4);
    opacity: 0;
    transform: translateY(6px) scale(0.9);
    transition: all 0.25s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .play :global(svg) {
    margin-left: 2px;
  }
  .cur .play,
  .card:hover .play {
    opacity: 1;
    transform: none;
  }
  .add {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(20, 12, 40, 0.45);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    opacity: 0;
    transition: opacity 0.2s;
  }
  .card:hover .add {
    opacity: 1;
  }
  .add:hover {
    background: rgba(20, 12, 40, 0.7);
  }
  .title {
    margin-top: 12px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .cur .title {
    color: #e9d5ff;
  }
  .artist {
    color: var(--muted);
    font-size: 13px;
    margin-top: 2px;
  }
</style>

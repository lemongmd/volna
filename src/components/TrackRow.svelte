<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Bars from './Bars.svelte';
  import { current, ps } from '../lib/state.svelte';
  import { addToQueue, playNext, playNow } from '../lib/sync';
  import type { Track } from '../lib/types';
  import { fmtTime } from '../lib/util';
  import { hoverEnd, hoverStart } from '../lib/prefetch';

  let { track, index }: { track: Track; index?: number } = $props();

  const isCur = $derived(current()?.id === track.id);
  const SRC: Record<string, string> = { ytm: 'YT Music', yt: 'YouTube', sc: 'SoundCloud', link: 'Ссылка' };
</script>

<div
  class="row"
  class:cur={isCur}
  ondblclick={() => playNow(track)}
  onmouseenter={() => hoverStart(track)}
  onmouseleave={hoverEnd}
  role="listitem"
>
  <div class="num">
    {#if isCur}
      <Bars playing={ps.playing} size={12} />
    {:else if index != null}
      <span class="n">{index + 1}</span>
    {/if}
    <button class="row-play" onclick={() => playNow(track)} aria-label="Играть"><Icon name="play" size={14} /></button>
  </div>
  <Cover src={track.thumb} size={40} radius={6} />
  <div class="meta">
    <div class="title">
      {#if track.explicit}<span class="badge-e" title="Explicit — без цензуры">E</span>{/if}
      <span class="ellipsis">{track.title}</span>
    </div>
    <div class="artist ellipsis">{track.artist}</div>
  </div>
  <span class="src">{SRC[track.source] ?? ''}</span>
  <div class="actions">
    <button class="icon-btn" onclick={() => playNext(track)} title="Сыграть следующим" aria-label="Сыграть следующим">
      <Icon name="playNext" size={16} />
    </button>
    <button class="icon-btn" onclick={() => addToQueue([track])} title="В очередь" aria-label="В очередь">
      <Icon name="plus" size={16} />
    </button>
  </div>
  <span class="dur">{track.duration ? fmtTime(track.duration) : ''}</span>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 28px 40px minmax(0, 1fr) 84px 72px 44px;
    align-items: center;
    gap: var(--s4);
    height: 58px;
    padding: 0 var(--s4) 0 var(--s2);
    border-radius: 10px;
    transition: background 0.12s;
  }
  .row:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .row.cur {
    background: rgba(255, 255, 255, 0.09);
  }
  .num {
    position: relative;
    display: grid;
    place-items: center;
    height: 28px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
  }
  .row-play {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: var(--text);
    opacity: 0;
  }
  .row:hover .row-play {
    opacity: 1;
  }
  .row:hover .n,
  .row:hover .num :global(.bars) {
    opacity: 0;
  }
  .meta {
    min-width: 0;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 550;
    min-width: 0;
  }
  .artist {
    color: var(--muted);
    font-size: 13px;
    margin-top: 2px;
  }
  .src {
    font-size: 11px;
    color: var(--faint);
    text-align: right;
  }
  .dur {
    color: var(--muted);
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 2px;
    opacity: 0;
  }
  .row:hover .actions {
    opacity: 1;
  }
</style>

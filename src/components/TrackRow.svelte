<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Bars from './Bars.svelte';
  import { current, ps } from '../lib/state.svelte';
  import { addToQueue, playNext, playNow } from '../lib/sync';
  import type { Track } from '../lib/types';
  import { fmtTime } from '../lib/util';

  let { track, index }: { track: Track; index?: number } = $props();

  const isCur = $derived(current()?.id === track.id);
  const SRC: Record<string, string> = { ytm: 'YT Music', yt: 'YouTube', sc: 'SoundCloud', link: 'Ссылка' };
</script>

<div class="row" class:cur={isCur} ondblclick={() => playNow(track)} role="listitem">
  <div class="num">
    {#if isCur}
      <Bars playing={ps.playing} size={14} />
    {:else if index != null}
      <span class="n">{index + 1}</span>
    {/if}
    <button class="row-play" onclick={() => playNow(track)} aria-label="Играть"><Icon name="play" size={16} /></button>
  </div>
  <Cover src={track.thumb} size={46} radius={10} />
  <div class="meta">
    <div class="title ellipsis">
      {#if track.explicit}<span class="badge-e" title="Без цензуры (explicit)">E</span>{/if}
      <span class="ellipsis">{track.title}</span>
    </div>
    <div class="artist ellipsis">{track.artist}</div>
  </div>
  <span class="src">{SRC[track.source] ?? ''}</span>
  <span class="dur">{track.duration ? fmtTime(track.duration) : ''}</span>
  <div class="actions">
    <button class="icon-btn" onclick={() => playNext(track)} title="Сыграть следующим" aria-label="Сыграть следующим">
      <Icon name="playNext" size={17} />
    </button>
    <button class="icon-btn" onclick={() => addToQueue([track])} title="В очередь" aria-label="В очередь">
      <Icon name="plus" size={18} />
    </button>
  </div>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 32px 46px minmax(0, 1fr) 90px 52px 84px;
    align-items: center;
    gap: 14px;
    padding: 7px 12px 7px 8px;
    border-radius: 14px;
    transition: background 0.15s;
  }
  .row:hover {
    background: rgba(255, 255, 255, 0.07);
  }
  .row.cur {
    background: linear-gradient(90deg, rgba(167, 139, 250, 0.22), rgba(167, 139, 250, 0.04));
  }
  .num {
    position: relative;
    display: grid;
    place-items: center;
    height: 32px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
  }
  .row-play {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: #fff;
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
    font-weight: 600;
  }
  .cur .title {
    color: #e9d5ff;
  }
  .artist {
    color: var(--muted);
    font-size: 13px;
    margin-top: 1px;
  }
  .src {
    font-size: 11px;
    color: var(--faint);
    text-align: right;
  }
  .dur {
    color: var(--muted);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .row:hover .actions {
    opacity: 1;
  }
</style>

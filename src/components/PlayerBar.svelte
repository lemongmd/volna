<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Slider from './Slider.svelte';
  import { current, pl, ps, room, settings, ui } from '../lib/state.svelte';
  import { dispatch } from '../lib/sync';
  import { fmtTime } from '../lib/util';

  const cur = $derived(current());
  const dur = $derived(pl.dur || cur?.duration || 0);
  const waiting = $derived(ps.playing && !!cur && (pl.loading || !ps.ready));

  let dragging = $state(false);
  let dragVal = $state(0);
  const shown = $derived(dragging ? dragVal : Math.min(pl.pos, dur || pl.pos));

  function cycleRepeat() {
    dispatch({ t: 'repeat', v: ps.repeat === 'off' ? 'all' : ps.repeat === 'all' ? 'one' : 'off' });
  }

  function togglePanel(tab: 'queue' | 'chat') {
    if (ui.panel && ui.tab === tab) ui.panel = false;
    else {
      ui.panel = true;
      ui.tab = tab;
      if (tab === 'chat') room.unread = 0;
    }
  }

  const volIcon = $derived(
    settings.muted || settings.volume === 0 ? 'mute' : settings.volume < 0.5 ? 'volumeLow' : 'volume',
  );
</script>

<footer class="player glass">
  <div class="track">
    {#if cur}
      <button class="cover-btn" onclick={() => (ui.view = ui.view === 'now' ? 'home' : 'now')} aria-label="Сейчас играет">
        <Cover src={cur.thumb} size={46} radius={8} />
        {#if pl.loading}<span class="cover-over"><span class="spinner"></span></span>{/if}
      </button>
      <div class="meta">
        <div class="title ellipsis">{cur.title}</div>
        <div class="artist ellipsis">
          {#if cur.explicit}<span class="badge-e">E</span>{/if}
          <span class="ellipsis">{cur.artist}{#if room.code && cur.by}<span class="by"> · {cur.by}</span>{/if}</span>
        </div>
      </div>
    {:else}
      <div class="empty muted">Ничего не играет</div>
    {/if}
  </div>

  <div class="center">
    <div class="controls">
      <button class="icon-btn" onclick={() => dispatch({ t: 'prev' })} aria-label="Назад" title="Назад (Ctrl+←)">
        <Icon name="prev" size={18} />
      </button>
      <button class="play" onclick={() => dispatch({ t: 'toggle' })} aria-label={ps.playing ? 'Пауза' : 'Играть'} title="Пробел">
        {#if waiting}
          <span class="spinner"></span>
        {:else if ps.playing}
          <Icon name="pause" size={16} />
        {:else}
          <Icon name="play" size={16} />
        {/if}
      </button>
      <button class="icon-btn" onclick={() => dispatch({ t: 'next' })} aria-label="Дальше" title="Дальше (Ctrl+→)">
        <Icon name="next" size={18} />
      </button>
    </div>
    <div class="progress">
      <span class="time">{fmtTime(shown)}</span>
      <Slider
        value={shown}
        max={dur || 1}
        step={0.1}
        disabled={!cur || !dur}
        label="Позиция трека"
        oninput={(v) => ((dragging = true), (dragVal = v))}
        onchange={(v) => ((dragging = false), dispatch({ t: 'seek', pos: v }))}
      />
      <span class="time">{fmtTime(dur)}</span>
    </div>
  </div>

  <div class="right">
    <div class="seg">
      <button class="icon-btn" onclick={() => dispatch({ t: 'shuffle' })} aria-label="Перемешать" title="Перемешать очередь">
        <Icon name="shuffle" size={16} />
      </button>
      <button
        class="icon-btn"
        class:on={ps.repeat !== 'off'}
        onclick={cycleRepeat}
        aria-label="Повтор"
        title={ps.repeat === 'one' ? 'Повтор трека' : ps.repeat === 'all' ? 'Повтор очереди' : 'Без повтора'}
      >
        <Icon name={ps.repeat === 'one' ? 'repeat1' : 'repeat'} size={16} />
      </button>
    </div>
    <div class="vol">
      <button class="icon-btn" onclick={() => (settings.muted = !settings.muted)} aria-label="Звук">
        <Icon name={volIcon} size={17} />
      </button>
      <Slider
        value={settings.muted ? 0 : settings.volume}
        max={1}
        step={0.01}
        label="Громкость"
        oninput={(v) => ((settings.volume = v), (settings.muted = false))}
      />
    </div>
    <div class="seg">
      {#if room.code}
        <button class="icon-btn rel" class:on={ui.panel && ui.tab === 'chat'} onclick={() => togglePanel('chat')} aria-label="Чат" title="Чат">
          <Icon name="chat" size={16} />
          {#if room.unread}<span class="unread">{room.unread > 9 ? '9+' : room.unread}</span>{/if}
        </button>
      {/if}
      <button class="icon-btn" class:on={ui.panel && ui.tab === 'queue'} onclick={() => togglePanel('queue')} aria-label="Очередь" title="Очередь">
        <Icon name="queue" size={16} />
      </button>
    </div>
  </div>
</footer>

<style>
  .player {
    flex: none;
    height: 66px;
    border-radius: var(--r-lg);
    display: grid;
    grid-template-columns: minmax(180px, 1fr) minmax(300px, 1.3fr) minmax(180px, 1fr);
    align-items: center;
    gap: 16px;
    padding: 0 10px;
    background: rgba(28, 28, 34, 0.62);
  }
  .track {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .cover-btn {
    position: relative;
    border-radius: 8px;
  }
  .cover-over {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
    border-radius: 8px;
  }
  .meta {
    min-width: 0;
  }
  .title {
    font-weight: 600;
  }
  .artist {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-top: 1px;
    font-size: 12.5px;
    color: var(--muted);
  }
  .by {
    color: var(--faint);
  }
  .empty {
    padding-left: 8px;
  }
  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .play {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--light);
    color: var(--light-ink);
    transition:
      transform 0.1s,
      background 0.15s;
  }
  .play:hover {
    background: #fff;
    transform: scale(1.05);
  }
  .play:active {
    transform: scale(0.95);
  }
  .play :global(svg) {
    margin-left: 1px;
  }
  .progress {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    max-width: 520px;
  }
  .time {
    font-size: 11px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
    min-width: 34px;
    text-align: center;
  }
  .right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }
  .seg {
    display: flex;
    gap: 2px;
    padding: 2px;
    border-radius: 10px;
    background: var(--panel);
  }
  .vol {
    display: flex;
    align-items: center;
    width: 130px;
  }
  .rel {
    position: relative;
  }
  .unread {
    position: absolute;
    top: 1px;
    right: 1px;
    min-width: 14px;
    height: 14px;
    padding: 0 3px;
    border-radius: 7px;
    background: var(--light);
    color: var(--light-ink);
    font-size: 9px;
    font-weight: 700;
    display: grid;
    place-items: center;
  }
</style>

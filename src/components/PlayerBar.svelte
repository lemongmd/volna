<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Bars from './Bars.svelte';
  import { current, pl, ps, room, settings, ui } from '../lib/state.svelte';
  import { dispatch } from '../lib/sync';
  import { fmtTime } from '../lib/util';

  const cur = $derived(current());
  const dur = $derived(pl.dur || cur?.duration || 0);

  let dragging = $state(false);
  let dragVal = $state(0);
  const shown = $derived(dragging ? dragVal : Math.min(pl.pos, dur || pl.pos));
  const pct = $derived(dur ? Math.min(100, (shown / dur) * 100) : 0);

  function onInput(e: Event) {
    dragging = true;
    dragVal = +(e.target as HTMLInputElement).value;
  }
  function onChange(e: Event) {
    const v = +(e.target as HTMLInputElement).value;
    dragging = false;
    dispatch({ t: 'seek', pos: v });
  }

  function cycleRepeat() {
    const next = ps.repeat === 'off' ? 'all' : ps.repeat === 'all' ? 'one' : 'off';
    dispatch({ t: 'repeat', v: next });
  }

  function togglePanel(tab: 'queue' | 'chat') {
    if (ui.panel && ui.tab === tab) ui.panel = false;
    else {
      ui.panel = true;
      ui.tab = tab;
      if (tab === 'chat') room.unread = 0;
    }
  }

  const volIcon = $derived(settings.muted || settings.volume === 0 ? 'mute' : settings.volume < 0.5 ? 'volumeLow' : 'volume');
</script>

<footer class="player glass liquid">
  <div class="track">
    {#if cur}
      <button class="cover-btn" onclick={() => (ui.view = ui.view === 'now' ? 'home' : 'now')} aria-label="Открыть плеер">
        <Cover src={cur.thumb} size={56} radius={14} />
        {#if pl.loading}
          <span class="cover-over"><span class="spinner"></span></span>
        {/if}
      </button>
      <div class="meta">
        <div class="title ellipsis">
          {#if cur.explicit}<span class="badge-e">E</span>{/if}
          {cur.title}
        </div>
        <div class="artist ellipsis">
          {#if ps.playing && !pl.loading && ps.ready}<Bars size={10} />{/if}
          {cur.artist}
          {#if room.code && cur.by}<span class="by">· от {cur.by}</span>{/if}
        </div>
      </div>
    {:else}
      <div class="empty">
        <Icon name="wave" size={22} />
        <span>Найди трек и нажми ▶</span>
      </div>
    {/if}
  </div>

  <div class="center">
    <div class="controls">
      <button class="icon-btn" onclick={() => dispatch({ t: 'shuffle' })} aria-label="Перемешать очередь" title="Перемешать">
        <Icon name="shuffle" size={17} />
      </button>
      <button class="icon-btn" onclick={() => dispatch({ t: 'prev' })} aria-label="Назад" title="Назад (Ctrl+←)">
        <Icon name="prev" size={20} />
      </button>
      <button class="play" onclick={() => dispatch({ t: 'toggle' })} aria-label={ps.playing ? 'Пауза' : 'Играть'} title="Пробел">
        {#if ps.playing && (pl.loading || (!ps.ready && !!cur))}
          <span class="spinner dark"></span>
        {:else if ps.playing}
          <Icon name="pause" size={22} />
        {:else}
          <Icon name="play" size={22} />
        {/if}
      </button>
      <button class="icon-btn" onclick={() => dispatch({ t: 'next' })} aria-label="Дальше" title="Дальше (Ctrl+→)">
        <Icon name="next" size={20} />
      </button>
      <button
        class="icon-btn"
        class:on={ps.repeat !== 'off'}
        onclick={cycleRepeat}
        aria-label="Повтор"
        title={ps.repeat === 'one' ? 'Повтор трека' : ps.repeat === 'all' ? 'Повтор очереди' : 'Без повтора'}
      >
        <Icon name={ps.repeat === 'one' ? 'repeat1' : 'repeat'} size={17} />
      </button>
    </div>
    <div class="progress">
      <span class="time">{fmtTime(shown)}</span>
      <div class="range" style="--p:{pct}%">
        <div class="fill"></div>
        <input
          type="range"
          min="0"
          max={dur || 1}
          step="0.1"
          value={shown}
          disabled={!cur || !dur}
          oninput={onInput}
          onchange={onChange}
          aria-label="Позиция трека"
        />
      </div>
      <span class="time">{fmtTime(dur)}</span>
    </div>
  </div>

  <div class="right">
    <button class="icon-btn" onclick={() => (settings.muted = !settings.muted)} aria-label="Звук">
      <Icon name={volIcon} size={18} />
    </button>
    <div class="range vol" style="--p:{(settings.muted ? 0 : settings.volume) * 100}%">
      <div class="fill"></div>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={settings.muted ? 0 : settings.volume}
        oninput={(e) => {
          settings.volume = +(e.target as HTMLInputElement).value;
          settings.muted = false;
        }}
        aria-label="Громкость"
      />
    </div>
    <span class="sep"></span>
    {#if room.code}
      <button class="icon-btn" class:on={ui.panel && ui.tab === 'chat'} onclick={() => togglePanel('chat')} aria-label="Чат">
        <Icon name="chat" size={18} />
        {#if room.unread}<span class="unread">{room.unread > 9 ? '9+' : room.unread}</span>{/if}
      </button>
    {/if}
    <button class="icon-btn" class:on={ui.panel && ui.tab === 'queue'} onclick={() => togglePanel('queue')} aria-label="Очередь">
      <Icon name="queue" size={18} />
    </button>
  </div>
</footer>

<style>
  .player {
    position: absolute;
    left: 16px;
    right: 16px;
    bottom: 16px;
    z-index: 30;
    height: 88px;
    border-radius: 26px;
    display: grid;
    grid-template-columns: minmax(200px, 1fr) minmax(360px, 1.4fr) minmax(200px, 1fr);
    align-items: center;
    gap: 20px;
    padding: 0 18px 0 16px;
    background: linear-gradient(135deg, rgba(42, 28, 74, 0.82), rgba(16, 11, 30, 0.8));
  }
  .track {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }
  .cover-btn {
    position: relative;
    border-radius: 14px;
    transition: transform 0.2s;
    box-shadow: 0 8px 22px rgba(0, 0, 0, 0.45);
  }
  .cover-btn:hover {
    transform: scale(1.04);
  }
  .cover-over {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
    border-radius: 14px;
  }
  .meta {
    min-width: 0;
  }
  .title {
    font-weight: 650;
    font-size: 15px;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .artist {
    color: var(--muted);
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
  }
  .by {
    color: var(--faint);
  }
  .empty {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--faint);
  }
  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .play {
    width: 46px;
    height: 46px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: #fff;
    color: #140c28;
    box-shadow:
      0 6px 24px rgba(192, 132, 252, 0.45),
      inset 0 -2px 0 rgba(0, 0, 0, 0.1);
    transition:
      transform 0.15s,
      box-shadow 0.2s;
  }
  .play:hover {
    transform: scale(1.06);
    box-shadow:
      0 8px 32px rgba(192, 132, 252, 0.7),
      inset 0 -2px 0 rgba(0, 0, 0, 0.1);
  }
  .play:active {
    transform: scale(0.94);
  }
  .play :global(svg) {
    margin-left: 1px;
  }
  .spinner.dark {
    border-color: rgba(20, 12, 40, 0.2);
    border-top-color: #140c28;
  }
  .progress {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    max-width: 560px;
  }
  .time {
    font-size: 11px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    min-width: 38px;
    text-align: center;
  }
  .range {
    position: relative;
    flex: 1;
    height: 18px;
    display: flex;
    align-items: center;
  }
  .range::before,
  .fill {
    content: '';
    position: absolute;
    left: 0;
    height: 4px;
    border-radius: 99px;
    pointer-events: none;
  }
  .range::before {
    right: 0;
    background: rgba(255, 255, 255, 0.14);
  }
  .fill {
    width: var(--p);
    background: linear-gradient(90deg, #a78bfa, #e9d5ff);
    box-shadow: 0 0 12px rgba(192, 132, 252, 0.6);
  }
  .range input {
    position: relative;
    width: 100%;
    margin: 0;
    height: 18px;
    background: transparent;
    appearance: none;
    cursor: pointer;
  }
  .range input::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.15), 0 2px 8px rgba(0, 0, 0, 0.4);
    opacity: 0;
    transition: opacity 0.15s;
  }
  .range:hover input::-webkit-slider-thumb {
    opacity: 1;
  }
  .range input:disabled {
    cursor: default;
  }
  .right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
  }
  .vol {
    flex: none;
    width: 100px;
  }
  .sep {
    width: 1px;
    height: 24px;
    background: var(--stroke);
    margin: 0 6px;
  }
  .right .icon-btn {
    position: relative;
  }
  .unread {
    position: absolute;
    top: 2px;
    right: 0;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 99px;
    background: #f472b6;
    color: #fff;
    font-size: 10px;
    font-weight: 700;
    display: grid;
    place-items: center;
  }
</style>

<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Avatar from './Avatar.svelte';
  import Slider from './Slider.svelte';
  import Lyrics from './Lyrics.svelte';
  import { current, pl, ps, room, ui } from '../lib/state.svelte';
  import { dispatch, sendReaction } from '../lib/sync';
  import { fmtTime } from '../lib/util';

  const cur = $derived(current());
  const dur = $derived(pl.dur || cur?.duration || 0);
  const upNext = $derived(ps.queue.slice(ps.index + 1, ps.index + 30));
  const waiting = $derived(ps.playing && !!cur && (pl.loading || !ps.ready));
  const EMOJI = ['🔥', '❤️', '😂', '😭', '🤯', '👏'];
  const SRC: Record<string, string> = { ytm: 'YouTube Music', yt: 'YouTube', sc: 'SoundCloud', link: 'По ссылке' };

  let dragging = $state(false);
  let dragVal = $state(0);
  const shown = $derived(dragging ? dragVal : Math.min(pl.pos, dur || pl.pos));
</script>

{#if cur}
  <div class="now">
    <div class="left">
      <div class="art"><Cover src={cur.thumb} size="100%" radius={12} /></div>
      <div class="title-row">
        <div class="meta">
          <h1 class="ellipsis" title={cur.title}>{cur.title}</h1>
          <div class="artist">
            {#if cur.explicit}<span class="badge-e">E</span>{/if}
            <span class="ellipsis">{cur.artist}</span>
          </div>
        </div>
        <span class="src">{SRC[cur.source] ?? ''}</span>
      </div>
      <div class="prog">
        <Slider
          value={shown}
          max={dur || 1}
          step={0.1}
          disabled={!dur}
          label="Позиция трека"
          oninput={(v) => ((dragging = true), (dragVal = v))}
          onchange={(v) => ((dragging = false), dispatch({ t: 'seek', pos: v }))}
        />
        <div class="times"><span>{fmtTime(shown)}</span><span>{fmtTime(dur)}</span></div>
      </div>
      <div class="controls">
        <button class="icon-btn" onclick={() => dispatch({ t: 'shuffle' })} aria-label="Перемешать"><Icon name="shuffle" size={18} /></button>
        <button class="icon-btn big" onclick={() => dispatch({ t: 'prev' })} aria-label="Назад"><Icon name="prev" size={22} /></button>
        <button class="play" onclick={() => dispatch({ t: 'toggle' })} aria-label={ps.playing ? 'Пауза' : 'Играть'}>
          {#if waiting}<span class="spinner"></span>{:else if ps.playing}<Icon name="pause" size={22} />{:else}<Icon name="play" size={22} />{/if}
        </button>
        <button class="icon-btn big" onclick={() => dispatch({ t: 'next' })} aria-label="Дальше"><Icon name="next" size={22} /></button>
        <button
          class="icon-btn"
          class:on={ps.repeat !== 'off'}
          onclick={() => dispatch({ t: 'repeat', v: ps.repeat === 'off' ? 'all' : ps.repeat === 'all' ? 'one' : 'off' })}
          aria-label="Повтор"
        >
          <Icon name={ps.repeat === 'one' ? 'repeat1' : 'repeat'} size={18} />
        </button>
      </div>
      <div class="status muted">
        {#if pl.loading}
          Загружаем трек…
        {:else if ps.playing && !ps.ready}
          Ждём хоста…
        {:else if pl.error}
          <span class="err">{pl.error}</span>
        {:else if room.code && cur.by}
          Поставлено: {cur.by}
        {/if}
      </div>
    </div>

    <div class="right">
      <div class="tabs">
        <button class="tab-chip" class:active={ui.npTab === 'lyrics'} onclick={() => (ui.npTab = 'lyrics')}>Текст</button>
        <button class="tab-chip" class:active={ui.npTab === 'next'} onclick={() => (ui.npTab = 'next')}>Дальше</button>
        {#if room.code}
          <button class="tab-chip" class:active={ui.npTab === 'room'} onclick={() => (ui.npTab = 'room')}>
            Слушают · {room.members.length}
          </button>
        {/if}
      </div>

      <div class="pane">
        {#if ui.npTab === 'lyrics'}
          <Lyrics />
        {:else if ui.npTab === 'room' && room.code}
          <section class="surface box">
            <div class="box-head">
              <span>Комната</span>
              <span class="mono muted">{room.code}</span>
            </div>
            <div class="people">
              {#each room.members as m (m.uid)}
                <span class="person">
                  <Avatar name={m.name} color={m.color} size={26} />
                  <span class="ellipsis">{m.name}</span>
                  {#if m.uid === ps.hostId}<span class="host" title="Хост"><Icon name="crown" size={11} /></span>{/if}
                </span>
              {/each}
            </div>
            <div class="reactions">
              {#each EMOJI as e}
                <button onclick={() => sendReaction(e)} aria-label="Реакция {e}">{e}</button>
              {/each}
            </div>
          </section>
        {:else}
          <section class="surface box scroll-box">
            <div class="box-head">
              <span>Дальше</span>
              <button class="link" onclick={() => ((ui.panel = true), (ui.tab = 'queue'))}>Вся очередь</button>
            </div>
            {#each upNext as q, i (q.qid)}
              <button class="next-row" onclick={() => dispatch({ t: 'jump', i: ps.index + 1 + i })}>
                <Cover src={q.thumb} size={36} radius={6} />
                <span class="nr-meta">
                  <span class="ellipsis nr-t">{q.title}</span>
                  <span class="ellipsis nr-a">{q.artist}</span>
                </span>
                <span class="nr-d">{q.duration ? fmtTime(q.duration) : ''}</span>
              </button>
            {:else}
              <p class="muted empty-q">Очередь закончится на этом треке.</p>
            {/each}
          </section>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <div class="empty">
    <Icon name="disc" size={44} stroke={1.4} />
    <h2>Сейчас ничего не играет</h2>
    <p class="muted">Найди трек в поиске или выбери подборку на главной.</p>
    <button class="btn btn-primary" onclick={() => (ui.view = 'search')}><Icon name="search" size={15} /> К поиску</button>
  </div>
{/if}

<style>
  .now {
    display: grid;
    grid-template-columns: minmax(240px, 360px) minmax(0, 1fr);
    gap: 48px;
    align-items: start;
    padding: 12px 8px 0;
    height: calc(100vh - 150px);
  }
  .left {
    display: flex;
    flex-direction: column;
  }
  .art {
    aspect-ratio: 1;
    border-radius: 12px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.45);
  }
  .title-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-top: 18px;
  }
  .meta {
    flex: 1;
    min-width: 0;
  }
  h1 {
    margin: 0;
    font-size: 21px;
    font-weight: 650;
    user-select: text;
  }
  .artist {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
    color: var(--muted);
  }
  .src {
    flex: none;
    font-size: 11px;
    color: var(--faint);
    padding: 4px 8px;
    border-radius: 6px;
    background: var(--panel);
    margin-top: 4px;
  }
  .prog {
    margin-top: 16px;
  }
  .times {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
    margin-top: 2px;
  }
  .controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 14px;
    margin-top: 8px;
  }
  .big {
    width: 40px;
    height: 40px;
    color: var(--text);
  }
  .play {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--light);
    color: var(--light-ink);
    transition: transform 0.1s;
  }
  .play:hover {
    transform: scale(1.04);
  }
  .play :global(svg) {
    margin-left: 2px;
  }
  .status {
    min-height: 20px;
    margin-top: 12px;
    text-align: center;
    font-size: 12.5px;
  }
  .err {
    color: var(--danger);
  }
  .right {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
    height: 100%;
    min-height: 0;
  }
  .tabs {
    display: flex;
    gap: 6px;
  }
  .pane {
    flex: 1;
    min-height: 0;
  }
  .scroll-box {
    max-height: 100%;
    overflow-y: auto;
  }
  .box {
    padding: 14px;
  }
  .box-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: 600;
    margin-bottom: 10px;
  }
  .people {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .person {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    max-width: 200px;
    padding: 3px 10px 3px 3px;
    border-radius: 99px;
    background: var(--panel-2);
    font-size: 13px;
  }
  .host {
    color: #f5c04e;
    display: grid;
  }
  .reactions {
    display: flex;
    gap: 4px;
    margin-top: 12px;
  }
  .reactions button {
    width: 38px;
    height: 38px;
    border-radius: 10px;
    font-size: 19px;
    background: var(--panel);
    transition: background 0.12s, transform 0.12s;
  }
  .reactions button:hover {
    background: var(--panel-3);
    transform: translateY(-2px);
  }
  .link {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--muted);
  }
  .link:hover {
    color: var(--text);
  }
  .next-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 5px 6px;
    border-radius: 8px;
    text-align: left;
  }
  .next-row:hover {
    background: var(--panel-2);
  }
  .nr-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .nr-t {
    font-weight: 550;
    font-size: 13px;
  }
  .nr-a,
  .nr-d {
    font-size: 12px;
    color: var(--muted);
  }
  .empty-q {
    margin: 4px 0;
    font-size: 13px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding-top: 14vh;
    color: var(--muted);
  }
  .empty h2 {
    margin: 16px 0 4px;
    color: var(--text);
    font-size: 19px;
  }
  .empty p {
    margin: 0 0 16px;
  }
  @container (max-width: 720px) {
    .now {
      grid-template-columns: 1fr;
      gap: 24px;
      height: auto;
    }
    .pane {
      height: 60vh;
    }
    .left {
      max-width: 380px;
    }
  }
</style>

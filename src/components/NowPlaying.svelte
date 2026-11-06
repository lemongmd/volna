<script lang="ts">
  import { onDestroy } from 'svelte';
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Avatar from './Avatar.svelte';
  import Slider from './Slider.svelte';
  import Lyrics from './Lyrics.svelte';
  import Aura from './Aura.svelte';
  import Capsule from './Capsule.svelte';
  import { current, pl, ps, room, settings, ui } from '../lib/state.svelte';
  import { dispatch, sendReaction } from '../lib/sync';
  import { currentTime } from '../lib/player';
  import { getLyrics, getOffset, type LyricsData } from '../lib/lyrics';
  import { fmtTime } from '../lib/util';

  const cur = $derived(current());
  const dur = $derived(pl.dur || cur?.duration || 0);
  const upNext = $derived(ps.queue.slice(ps.index + 1, ps.index + 30));
  const waiting = $derived(ps.playing && !!cur && (pl.loading || !ps.ready));
  const EMOJI = ['🔥', '❤️', '😂', '😭', '🤯', '👏'];
  const SRC: Record<string, string> = { ytm: 'YouTube Music', yt: 'YouTube', sc: 'SoundCloud', link: 'По ссылке' };

  const context = $derived(
    room.code ? `Комната ${room.code} · слушают ${room.members.length}` : cur ? (SRC[cur.source] ?? 'Сейчас играет') : '',
  );
  const status = $derived(
    pl.loading ? 'Загружаем трек…' : ps.playing && !ps.ready ? 'Ждём хоста…' : pl.error ? pl.error : '',
  );

  const volIcon = $derived(
    settings.muted || settings.volume === 0 ? 'mute' : settings.volume < 0.5 ? 'volumeLow' : 'volume',
  );
  let volOpen = $state(false);

  // Очередь и чат открываются в боковой панели
  function togglePanel(tab: 'queue' | 'chat') {
    if (ui.panel && ui.tab === tab) ui.panel = false;
    else {
      ui.panel = true;
      ui.tab = tab;
      if (tab === 'chat') room.unread = 0;
    }
  }

  function cycleRepeat() {
    dispatch({ t: 'repeat', v: ps.repeat === 'off' ? 'all' : ps.repeat === 'all' ? 'one' : 'off' });
  }

  // Текущая строка песни под кнопками на сцене
  let lyr = $state<LyricsData | null>(null);
  let lyrFor = '';
  let line = $state('');
  let raf = 0;
  $effect(() => {
    const t = cur;
    if (!t || t.id === lyrFor) return;
    lyrFor = t.id;
    lyr = null;
    line = '';
    getLyrics(t)
      .then((d) => {
        if (lyrFor === t.id) lyr = d;
      })
      .catch(() => {});
  });
  function tick() {
    raf = requestAnimationFrame(tick);
    const lines = lyr?.lines;
    if (!lines?.length || !cur || ui.npTab !== 'stage') return;
    const now = currentTime() + getOffset(cur);
    let txt = '';
    for (const l of lines) {
      if (l.t <= now) txt = l.text;
      else break;
    }
    if (txt !== line) line = txt;
  }
  raf = requestAnimationFrame(tick);
  onDestroy(() => cancelAnimationFrame(raf));

  let coverEl: HTMLDivElement | undefined = $state();

  const TABS = [
    { id: 'stage', label: 'Сцена' },
    { id: 'lyrics', label: 'Текст' },
    { id: 'next', label: 'Дальше' },
  ] as const;
</script>

{#snippet transport(big: boolean)}
  <div class="transport" class:big>
    <button class="ring" onclick={() => dispatch({ t: 'shuffle' })} aria-label="Перемешать" title="Перемешать очередь">
      <Icon name="shuffle" size={18} />
    </button>
    <button class="ring" onclick={() => dispatch({ t: 'prev' })} aria-label="Назад" title="Назад">
      <Icon name="prev" size={18} />
    </button>
    <button class="play" onclick={() => dispatch({ t: 'toggle' })} aria-label={ps.playing ? 'Пауза' : 'Играть'} title="Пробел">
      {#if waiting}<span class="spinner"></span>{:else if ps.playing}<Icon name="pause" size={big ? 30 : 24} />{:else}<Icon name="play" size={big ? 30 : 24} />{/if}
    </button>
    <button class="ring" onclick={() => dispatch({ t: 'next' })} aria-label="Дальше" title="Дальше">
      <Icon name="next" size={18} />
    </button>
    <button
      class="ring"
      class:on={ps.repeat !== 'off'}
      onclick={cycleRepeat}
      aria-label="Повтор"
      title={ps.repeat === 'one' ? 'Повтор трека' : ps.repeat === 'all' ? 'Повтор очереди' : 'Без повтора'}
    >
      <Icon name={ps.repeat === 'one' ? 'repeat1' : 'repeat'} size={18} />
    </button>
  </div>
{/snippet}

{#snippet tools()}
  <div class="vol-wrap">
    <button class="ring" class:on={volOpen} onclick={() => (volOpen = !volOpen)} aria-label="Громкость" title="Громкость" aria-expanded={volOpen}>
      <Icon name={volIcon} size={18} />
    </button>
    {#if volOpen}
      <div class="vol-pop">
        <button class="icon-btn" onclick={() => (settings.muted = !settings.muted)} aria-label={settings.muted ? 'Включить звук' : 'Выключить звук'}>
          <Icon name={volIcon} size={16} />
        </button>
        <Slider
          value={settings.muted ? 0 : settings.volume}
          max={1}
          step={0.01}
          label="Громкость"
          oninput={(v) => ((settings.volume = v), (settings.muted = false))}
        />
      </div>
    {/if}
  </div>
{/snippet}

{#snippet sideTools()}
  <button class="ring" class:on={ui.panel && ui.tab === 'queue'} onclick={() => togglePanel('queue')} aria-label="Очередь" title="Очередь">
    <Icon name="queue" size={18} />
  </button>
  {#if room.code}
    <button class="ring rel" class:on={ui.panel && ui.tab === 'chat'} onclick={() => togglePanel('chat')} aria-label="Чат" title="Чат">
      <Icon name="chat" size={18} />
      {#if room.unread}<span class="unread">{room.unread > 9 ? '9+' : room.unread}</span>{/if}
    </button>
  {/if}
{/snippet}

{#if cur}
  <div class="np" class:stage-mode={ui.npTab === 'stage'}>
    <div class="tabs" role="tablist" aria-label="Вид экрана">
      {#each TABS as t}
        <button class="tab-chip" class:active={ui.npTab === t.id} role="tab" aria-selected={ui.npTab === t.id} onclick={() => (ui.npTab = t.id)}>
          {t.label}
        </button>
      {/each}
      {#if room.code}
        <button class="tab-chip" class:active={ui.npTab === 'room'} role="tab" aria-selected={ui.npTab === 'room'} onclick={() => (ui.npTab = 'room')}>
          Слушают · {room.members.length}
        </button>
      {/if}
    </div>

    {#if ui.npTab === 'stage'}
      <!-- Сцена: всё по центру, крупное имя артиста, аура вокруг обложки -->
      <div class="stage">
        <Aura target={coverEl} playing={ps.playing && ps.ready} />
        <span class="pill">{context}</span>
        <h1 class="artist display" title={cur.artist}>{cur.artist || cur.title}</h1>
        <div class="cover" bind:this={coverEl}><Cover src={cur.thumb} size="100%" radius={16} /></div>
        <div class="track">
          {#if cur.explicit}<span class="badge-e">E</span>{/if}
          <span class="ellipsis">{cur.title}</span>
        </div>
        <div class="row">
          {@render tools()}
          <button class="ring" onclick={() => (ui.npTab = 'lyrics')} aria-label="Текст песни" title="Текст песни">
            <Icon name="lyrics" size={18} />
          </button>
          <Capsule pos={pl.pos} {dur} label={`${cur.artist} — ${cur.title}`} disabled={!dur} onseek={(v) => dispatch({ t: 'seek', pos: v })} />
          {@render sideTools()}
        </div>
        {@render transport(true)}
        {#if status}
          <p class="status" class:err={!!pl.error}>{status}</p>
        {:else if line}
          <button class="line display" onclick={() => (ui.npTab = 'lyrics')} title="Открыть текст">{line}</button>
        {:else if room.code && cur.by}
          <p class="status">Поставлено: {cur.by}</p>
        {/if}
      </div>
    {:else}
      <!-- Текст / Дальше / Слушают: обложка с управлением слева, содержимое справа -->
      <div class="split">
        <div class="left">
          <div class="art"><Cover src={cur.thumb} size="100%" radius={16} /></div>
          <div class="meta">
            <h2 class="name display ellipsis" title={cur.title}>{cur.title}</h2>
            <div class="artist-sm">
              {#if cur.explicit}<span class="badge-e">E</span>{/if}
              <span class="ellipsis">{cur.artist}</span>
            </div>
          </div>
          <Capsule pos={pl.pos} {dur} label={context} disabled={!dur} onseek={(v) => dispatch({ t: 'seek', pos: v })} />
          {@render transport(false)}
          <div class="row compact">
            {@render tools()}
            <span class="grow"></span>
            {@render sideTools()}
          </div>
          {#if status}<p class="status" class:err={!!pl.error}>{status}</p>{/if}
        </div>

        <div class="pane">
          {#if ui.npTab === 'lyrics'}
            <Lyrics />
          {:else if ui.npTab === 'room' && room.code}
            <section class="box">
              <div class="box-head">
                <span>Комната</span>
                <span class="mono muted">{room.code}</span>
              </div>
              <div class="people">
                {#each room.members as m (m.uid)}
                  <span class="person">
                    <Avatar name={m.name} color={m.color} size={32} />
                    <span class="ellipsis">{m.name}</span>
                    {#if m.uid === ps.hostId}<span class="host" title="Хост"><Icon name="crown" size={12} /></span>{/if}
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
            <section class="box scroll-box">
              <div class="box-head">
                <span>Дальше</span>
                <button class="link" onclick={() => ((ui.panel = true), (ui.tab = 'queue'))}>Вся очередь</button>
              </div>
              {#each upNext as q, i (q.qid)}
                <button class="next-row" onclick={() => dispatch({ t: 'jump', i: ps.index + 1 + i })}>
                  <Cover src={q.thumb} size={48} radius={8} />
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
    {/if}
  </div>
{:else}
  <div class="empty">
    <Icon name="nowPlaying" size={48} />
    <h2>Сейчас ничего не играет</h2>
    <p class="muted">Найди трек в поиске или выбери подборку на главной.</p>
    <button class="btn btn-primary" onclick={() => (ui.view = 'search')}><Icon name="search" size={15} /> К поиску</button>
  </div>
{/if}

<style>
  .np {
    display: flex;
    flex-direction: column;
    gap: var(--s4);
    /* окно минус отступы main и колонки: нижней панели на этом экране нет */
    height: calc(100vh - var(--s7) - var(--s6) - var(--s4));
    min-height: 560px;
  }
  .tabs {
    display: flex;
    justify-content: center;
    gap: var(--s2);
    flex-wrap: wrap;
  }

  /* ───── сцена ───── */
  .stage {
    position: relative;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: safe center;
    gap: var(--s4);
    isolation: isolate;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: none;
  }
  .stage > :not(:global(.aura)) {
    position: relative;
    z-index: 1;
    flex-shrink: 0;
  }
  .pill {
    padding: var(--s1) var(--s4);
    border-radius: var(--pill);
    border: 1px solid rgba(255, 255, 255, 0.35);
    background: rgba(7, 7, 8, 0.4);
    font-size: 13px;
    font-weight: 700;
  }
  .artist {
    margin: 0;
    max-width: 100%;
    /* размер зависит и от ширины, и от высоты окна */
    font-size: clamp(32px, min(7.2vw, 10vh), 108px);
    line-height: 0.95;
    text-align: center;
    text-wrap: balance;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    padding-bottom: 0.06em;
    user-select: text;
  }
  .cover {
    width: clamp(160px, 30vh, 280px);
    aspect-ratio: 1;
    max-width: 100%;
    border-radius: 16px;
    box-shadow: 0 30px 80px color-mix(in srgb, var(--cover) 28%, transparent);
  }
  .track {
    display: flex;
    align-items: center;
    gap: var(--s2);
    max-width: 100%;
    font-size: 22px;
    font-weight: 800;
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--s2);
    width: min(720px, 100%);
  }
  .transport {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s4);
  }
  .play {
    width: 64px;
    height: 64px;
    border-radius: var(--pill);
    display: grid;
    place-items: center;
    border: 3px solid var(--text);
    color: var(--text);
    transition:
      transform 0.1s,
      background 0.15s;
  }
  .transport.big .play {
    width: 80px;
    height: 80px;
  }
  .play:hover {
    background: var(--panel-2);
    transform: scale(1.04);
  }
  .play:active {
    transform: scale(0.96);
  }
  .play :global(svg) {
    margin-left: 2px;
  }
  .line {
    max-width: min(760px, 100%);
    min-height: 1.3em;
    font-size: 24px;
    line-height: 1.25;
    text-align: center;
    text-wrap: balance;
    color: rgba(255, 255, 255, 0.86);
    animation: fade-in 0.35s ease both;
  }
  .line:hover {
    color: #fff;
  }
  .status {
    margin: 0;
    min-height: 1.3em;
    color: var(--muted);
    font-size: 14px;
    text-align: center;
  }
  .status.err {
    color: var(--danger);
  }

  .vol-wrap {
    position: relative;
  }
  .vol-pop {
    position: absolute;
    bottom: calc(100% + var(--s2));
    left: 0;
    z-index: 5;
    display: flex;
    align-items: center;
    gap: var(--s2);
    width: 220px;
    padding: var(--s2) var(--s4) var(--s2) var(--s2);
    border-radius: var(--pill);
    background: rgba(24, 24, 28, 0.96);
    border: 1px solid var(--stroke-2);
    animation: fade-in 0.15s ease both;
  }
  .rel {
    position: relative;
  }
  .unread {
    position: absolute;
    top: -2px;
    right: -2px;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    border-radius: var(--pill);
    background: var(--cover);
    color: var(--light-ink);
    font-size: 10px;
    font-weight: 800;
    display: grid;
    place-items: center;
  }

  /* ───── две колонки ───── */
  .split {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(260px, 360px) minmax(0, 640px);
    justify-content: center;
    gap: var(--s7);
  }
  .left {
    display: flex;
    flex-direction: column;
    gap: var(--s4);
    min-width: 0;
  }
  .art {
    aspect-ratio: 1;
    border-radius: 16px;
    box-shadow: 0 24px 60px color-mix(in srgb, var(--cover) 24%, transparent);
  }
  .meta {
    min-width: 0;
  }
  .name {
    margin: 0;
    font-size: 30px;
    line-height: 1.05;
  }
  .artist-sm {
    display: flex;
    align-items: center;
    gap: var(--s2);
    margin-top: var(--s1);
    color: var(--muted);
    font-size: 15px;
    font-weight: 600;
  }
  .left .transport {
    gap: var(--s2);
  }
  /* в колонке капсула не должна растягиваться по высоте */
  .left :global(.capsule) {
    flex: none;
  }
  /* на сцене под кольцами и капсулой тёмная подложка, чтобы аура не мешала читать */
  .stage :global(.ring) {
    background: rgba(7, 7, 8, 0.55);
  }
  .row.compact {
    width: 100%;
  }
  .grow {
    flex: 1;
  }
  .pane {
    min-height: 0;
    min-width: 0;
  }
  .box {
    padding: var(--s5);
    border-radius: var(--r-xl);
    background: var(--panel);
    border: 1px solid var(--stroke);
  }
  .scroll-box {
    max-height: 100%;
    overflow-y: auto;
  }
  .box-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: 700;
    margin-bottom: var(--s4);
  }
  .people {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s2);
  }
  .person {
    display: inline-flex;
    align-items: center;
    gap: var(--s2);
    max-width: 220px;
    height: 40px;
    padding: 0 var(--s4) 0 var(--s1);
    border-radius: var(--pill);
    background: var(--panel-2);
    font-weight: 600;
  }
  .host {
    color: #f5c04e;
    display: grid;
  }
  .reactions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s2);
    margin-top: var(--s4);
  }
  .reactions button {
    width: 48px;
    height: 48px;
    border-radius: var(--pill);
    font-size: 22px;
    border: 1px solid var(--stroke-2);
    transition:
      background 0.12s,
      transform 0.12s;
  }
  .reactions button:hover {
    background: var(--panel-2);
    transform: translateY(-2px);
  }
  .link {
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }
  .link:hover {
    color: var(--text);
  }
  .next-row {
    display: flex;
    align-items: center;
    gap: var(--s3);
    width: 100%;
    padding: var(--s2);
    border-radius: var(--r);
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
    font-weight: 700;
  }
  .nr-a,
  .nr-d {
    font-size: 13px;
    color: var(--muted);
  }
  .empty-q {
    margin: var(--s2) 0;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--s2);
    padding-top: 16vh;
    color: var(--muted);
  }
  .empty h2 {
    margin: var(--s4) 0 0;
    color: var(--text);
    font-size: 32px;
  }
  .empty p {
    margin: 0 0 var(--s4);
  }

  /* невысокое окно: сцена плотнее */
  @media (max-height: 760px) {
    .stage {
      gap: var(--s2);
    }
    .transport.big .play {
      width: 64px;
      height: 64px;
    }
    .line {
      font-size: 20px;
    }
  }

  /* узкое окно / телефон */
  @container (max-width: 760px) {
    .np {
      height: auto;
      min-height: 0;
    }
    .stage {
      padding-block: var(--s5);
    }
    .row {
      flex-wrap: wrap;
      justify-content: center;
    }
    .row :global(.capsule) {
      order: -1;
      flex-basis: 100%;
    }
    .transport,
    .transport.big {
      gap: var(--s2);
    }
    .transport :global(.ring) {
      width: 40px;
      height: 40px;
    }
    .transport.big .play {
      width: 56px;
      height: 56px;
    }
    .split {
      grid-template-columns: 1fr;
      gap: var(--s5);
    }
    .left {
      max-width: 380px;
      width: 100%;
      justify-self: center;
    }
    .pane {
      height: 60vh;
    }
  }
</style>

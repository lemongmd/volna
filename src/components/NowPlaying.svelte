<script lang="ts">
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Avatar from './Avatar.svelte';
  import Bars from './Bars.svelte';
  import { current, pl, ps, room, ui } from '../lib/state.svelte';
  import { dispatch, sendReaction } from '../lib/sync';
  import { fmtTime } from '../lib/util';

  const cur = $derived(current());
  const upNext = $derived(ps.queue.slice(ps.index + 1, ps.index + 6));
  const EMOJI = ['🔥', '💜', '😂', '😭', '🤯', '👏'];
  const SRC: Record<string, string> = { ytm: 'YouTube Music', yt: 'YouTube', sc: 'SoundCloud', link: 'По ссылке' };
</script>

{#if cur}
  <div class="now">
    <div class="art-wrap">
      <div class="art-glow" style="background-image:url('{cur.thumb}')"></div>
      <div class="art" class:spin={ps.playing && ps.ready}>
        <Cover src={cur.thumb} size="100%" radius={32} />
        <div class="art-shine"></div>
      </div>
    </div>

    <div class="info">
      <div class="src-pill">
        {#if ps.playing && ps.ready}<Bars size={12} />{/if}
        {SRC[cur.source] ?? ''}
        {#if room.code}· комната {room.code}{/if}
      </div>
      <h1>
        {#if cur.explicit}<span class="badge-e big">E</span>{/if}
        {cur.title}
      </h1>
      <p class="artist">{cur.artist}</p>
      {#if cur.by && room.code}<p class="muted by">Трек в очереди от {cur.by}</p>{/if}

      <div class="status">
        {#if pl.loading}
          <span class="spinner"></span> Загружаем трек…
        {:else if ps.playing && !ps.ready}
          <span class="spinner"></span> Ждём, пока хост начнёт…
        {:else if pl.error}
          <span class="err">{pl.error}</span>
        {:else}
          {fmtTime(pl.pos)} / {fmtTime(pl.dur || cur.duration)}
        {/if}
      </div>

      {#if room.code}
        <div class="listeners">
          <span class="muted">Слушают сейчас</span>
          <div class="stack">
            {#each room.members as m (m.uid)}
              <span class="li">
                <Avatar name={m.name} color={m.color} size={34} />
                {#if m.uid === ps.hostId}<span class="crown"><Icon name="crown" size={11} stroke={2.4} /></span>{/if}
              </span>
            {/each}
          </div>
        </div>
      {/if}

      <div class="reactions glass">
        {#each EMOJI as e}
          <button onclick={() => sendReaction(e)} aria-label="Реакция {e}">{e}</button>
        {/each}
      </div>

      {#if upNext.length}
        <div class="next">
          <div class="next-head">
            <span class="muted">Дальше в очереди</span>
            <button class="link" onclick={() => ((ui.panel = true), (ui.tab = 'queue'))}>Вся очередь</button>
          </div>
          {#each upNext as q, i (q.qid)}
            <button class="next-row" onclick={() => dispatch({ t: 'jump', i: ps.index + 1 + i })}>
              <Cover src={q.thumb} size={40} radius={9} />
              <span class="nr-meta">
                <span class="ellipsis nr-t">{q.title}</span>
                <span class="ellipsis muted nr-a">{q.artist}</span>
              </span>
              <span class="muted nr-d">{q.duration ? fmtTime(q.duration) : ''}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{:else}
  <div class="empty">
    <div class="disc"><Icon name="disc" size={56} stroke={1.4} /></div>
    <h2>Сейчас ничего не играет</h2>
    <p class="muted">Найди трек в поиске или выбери подборку на главной.</p>
    <button class="btn btn-primary" onclick={() => (ui.view = 'search')}><Icon name="search" size={16} /> К поиску</button>
  </div>
{/if}

<style>
  .now {
    display: grid;
    grid-template-columns: minmax(280px, 440px) minmax(0, 1fr);
    gap: 56px;
    align-items: center;
    padding: 28px 12px 0;
    min-height: calc(100vh - 230px);
  }
  .art-wrap {
    position: relative;
  }
  @container (max-width: 760px) {
    .now {
      grid-template-columns: 1fr;
      gap: 28px;
    }
    .art-wrap {
      max-width: 320px;
    }
  }
  .art-glow {
    position: absolute;
    inset: 8%;
    background-size: cover;
    background-position: center;
    filter: blur(60px) saturate(1.8);
    opacity: 0.8;
    border-radius: 50%;
  }
  .art {
    position: relative;
    aspect-ratio: 1;
    border-radius: 32px;
    box-shadow:
      0 40px 80px rgba(0, 0, 0, 0.55),
      0 0 0 1px rgba(255, 255, 255, 0.1);
    transition: transform 0.6s cubic-bezier(0.2, 0.8, 0.2, 1);
    transform: scale(0.94);
  }
  .art.spin {
    transform: scale(1);
  }
  .art-shine {
    position: absolute;
    inset: 0;
    border-radius: 32px;
    background: linear-gradient(155deg, rgba(255, 255, 255, 0.28), transparent 38%, transparent 70%, rgba(255, 255, 255, 0.08));
    pointer-events: none;
  }
  .info {
    min-width: 0;
  }
  .src-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--stroke);
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  h1 {
    font-size: 46px;
    line-height: 1.05;
    letter-spacing: -0.03em;
    margin: 18px 0 8px;
    font-weight: 750;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    user-select: text;
  }
  .badge-e.big {
    font-size: 14px;
    height: 26px;
    min-width: 26px;
    vertical-align: middle;
    border-radius: 6px;
    margin-right: 6px;
  }
  .artist {
    font-size: 20px;
    color: rgba(255, 255, 255, 0.78);
    margin: 0;
  }
  .by {
    margin: 6px 0 0;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 18px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .err {
    color: var(--danger);
  }
  .listeners {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 22px;
  }
  .stack {
    display: flex;
  }
  .li {
    position: relative;
    margin-right: -8px;
    border-radius: 50%;
    outline: 3px solid #0d0a18;
  }
  .crown {
    position: absolute;
    top: -9px;
    left: 50%;
    transform: translateX(-50%);
    color: #fbbf24;
    filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.6));
  }
  .reactions {
    display: inline-flex;
    gap: 4px;
    margin-top: 24px;
    padding: 6px;
    border-radius: 999px;
  }
  .reactions button {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    font-size: 22px;
    transition: transform 0.15s, background 0.15s;
  }
  .reactions button:hover {
    background: rgba(255, 255, 255, 0.12);
    transform: scale(1.18) translateY(-2px);
  }
  .reactions button:active {
    transform: scale(0.9);
  }
  .next {
    margin-top: 28px;
    max-width: 520px;
  }
  .next-head {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 13px;
  }
  .link {
    color: var(--accent);
    font-weight: 600;
  }
  .next-row {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 6px;
    border-radius: 12px;
    text-align: left;
  }
  .next-row:hover {
    background: rgba(255, 255, 255, 0.07);
  }
  .nr-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .nr-t {
    font-weight: 600;
  }
  .nr-a,
  .nr-d {
    font-size: 12px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding-top: 12vh;
    gap: 6px;
  }
  .empty h2 {
    margin: 18px 0 0;
  }
  .empty p {
    margin: 0 0 16px;
  }
  .disc {
    width: 120px;
    height: 120px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: rgba(255, 255, 255, 0.7);
    background: radial-gradient(circle at 30% 25%, rgba(255, 255, 255, 0.35), transparent 35%),
      conic-gradient(from 0deg, #4c1d95, #7c3aed, #c084fc, #38bdf8, #4c1d95);
    box-shadow: 0 30px 70px rgba(124, 58, 237, 0.4);
  }
  :global(.fx) .disc {
    animation: spin 12s linear infinite;
  }
</style>

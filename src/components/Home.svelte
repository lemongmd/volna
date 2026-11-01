<script lang="ts">
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';
  import TrackCard from './TrackCard.svelte';
  import { profile, recent, room, ui } from '../lib/state.svelte';
  import { PICKS, searchCached } from '../lib/catalog';
  import { errText, isLink } from '../lib/util';
  import type { Track } from '../lib/types';

  let pick = $state(0);
  let items = $state<Track[]>([]);
  let loading = $state(false);
  let error = $state('');
  let link = $state('');

  const hour = new Date().getHours();
  const hello = hour < 6 ? 'Доброй ночи' : hour < 12 ? 'Доброе утро' : hour < 18 ? 'Добрый день' : 'Добрый вечер';

  async function loadPick(i: number) {
    pick = i;
    loading = true;
    error = '';
    try {
      const res = await searchCached(PICKS[i].query, 'ytm');
      if (pick === i) items = res.slice(0, 18);
    } catch (e) {
      if (pick === i) error = errText(e);
    } finally {
      if (pick === i) loading = false;
    }
  }

  function openLink() {
    const v = link.trim();
    if (!v) return;
    ui.query = v;
    ui.view = 'search';
  }

  async function copyCode() {
    if (!room.code) return;
    await navigator.clipboard.writeText(room.code).catch(() => {});
  }

  $effect(() => {
    if (ui.ready && !items.length && !loading && !error) loadPick(pick);
  });
</script>

<section class="hello">
  <h1>{hello}{profile.name ? `, ${profile.name}` : ''}</h1>
  <p class="muted">Что слушаем сегодня?</p>
</section>

<section class="hero">
  <div class="together glass">
    <div class="glow"></div>
    {#if room.code}
      <div class="label"><span class="live-dot"></span> Вы в комнате</div>
      <div class="big-code">
        {room.code}
        <button class="icon-btn" onclick={copyCode} title="Скопировать код" aria-label="Скопировать код">
          <Icon name="copy" size={18} />
        </button>
      </div>
      <p class="muted">Друзья вводят этот код и слышат то же, что и ты — секунда в секунду.</p>
      <div class="people">
        {#each room.members as m (m.uid)}
          <span class="person"><Avatar name={m.name} color={m.color} size={28} /> {m.name}</span>
        {/each}
      </div>
      <div class="actions">
        <button class="btn btn-primary" onclick={() => ((ui.panel = true), (ui.tab = 'chat'))}>
          <Icon name="chat" size={16} /> Открыть чат
        </button>
        <button class="btn btn-glass" onclick={() => ((ui.panel = true), (ui.tab = 'queue'))}>Общая очередь</button>
      </div>
    {:else}
      <div class="label"><Icon name="sparkles" size={14} /> Мультиплеер</div>
      <h2>Слушайте<br />вместе</h2>
      <p class="muted">Создай комнату, скинь код другу — и вы слушаете одну и ту же музыку одновременно. С чатом и реакциями.</p>
      <div class="actions">
        <button class="btn btn-primary" onclick={() => (ui.modal = 'join')}>
          Создать комнату <Icon name="arrowRight" size={16} />
        </button>
        <button class="btn btn-glass" onclick={() => (ui.modal = 'join')}>Войти по коду</button>
      </div>
    {/if}
    {#if !room.code}<div class="orb"></div>{/if}
  </div>

  <div class="paste glass">
    <div class="label"><Icon name="link" size={14} /> Вставь ссылку</div>
    <h3>YouTube, SoundCloud, Spotify и ещё 1000+ сайтов</h3>
    <p class="muted">Трек, альбом или плейлист — Волна найдёт и сыграет. Треки из Spotify ищутся на YouTube без цензуры.</p>
    <form
      class="paste-form"
      onsubmit={(e) => {
        e.preventDefault();
        openLink();
      }}
    >
      <input class="input" placeholder="https://…" bind:value={link} spellcheck="false" />
      <button class="btn btn-accent" type="submit" disabled={!isLink(link)}>Открыть</button>
    </form>
  </div>
</section>

{#if recent.length}
  <section class="block">
    <div class="block-head">
      <h2 class="section-title">Недавно слушали</h2>
    </div>
    <div class="scroller">
      {#each recent.slice(0, 14) as t (t.id)}
        <div class="sc-item"><TrackCard track={t} /></div>
      {/each}
    </div>
  </section>
{/if}

<section class="block">
  <div class="block-head">
    <h2 class="section-title">Подборки</h2>
  </div>
  <div class="chips">
    {#each PICKS as p, i}
      <button class="chip" class:active={pick === i} onclick={() => loadPick(i)}>{p.emoji} {p.label}</button>
    {/each}
  </div>
  {#if error}
    <div class="err">
      <p>Не получилось загрузить подборку: {error}</p>
      <button class="btn btn-glass" onclick={() => loadPick(pick)}><Icon name="refresh" size={16} /> Ещё раз</button>
    </div>
  {:else}
    <div class="grid">
      {#if loading && !items.length}
        {#each Array(12) as _}
          <div><div class="sk-art skeleton"></div><div class="sk-line skeleton"></div><div class="sk-line short skeleton"></div></div>
        {/each}
      {:else}
        {#each items as t (t.id)}
          <TrackCard track={t} />
        {/each}
      {/if}
    </div>
  {/if}
</section>

<style>
  .hello h1 {
    margin: 18px 0 2px;
    font-size: 34px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .hello p {
    margin: 0 0 22px;
    font-size: 15px;
  }
  .hero {
    display: grid;
    grid-template-columns: 1.35fr 1fr;
    gap: 20px;
  }
  .together,
  .paste {
    border-radius: var(--r-lg);
    padding: 28px;
    overflow: hidden;
    min-height: 260px;
  }
  .together {
    background: linear-gradient(140deg, rgba(124, 58, 237, 0.55), rgba(76, 29, 149, 0.35) 45%, rgba(14, 165, 233, 0.18));
  }
  .glow {
    position: absolute;
    inset: auto -20% -60% 30%;
    height: 120%;
    background: radial-gradient(circle, rgba(192, 132, 252, 0.45), transparent 60%);
    pointer-events: none;
    z-index: -1;
  }
  .orb {
    position: absolute;
    right: 34px;
    top: 50%;
    width: 170px;
    height: 170px;
    margin-top: -85px;
    border-radius: 50%;
    background:
      radial-gradient(circle at 32% 28%, rgba(255, 255, 255, 0.85), rgba(255, 255, 255, 0) 22%),
      radial-gradient(circle at 50% 50%, rgba(196, 181, 253, 0.55), rgba(124, 58, 237, 0.35) 55%, rgba(56, 189, 248, 0.35));
    box-shadow:
      inset 0 0 40px rgba(255, 255, 255, 0.25),
      inset -10px -20px 50px rgba(30, 10, 60, 0.5),
      0 30px 60px rgba(76, 29, 149, 0.5);
    pointer-events: none;
    z-index: -1;
  }
  :global(.fx) .orb {
    animation: bob 6s ease-in-out infinite alternate;
  }
  @keyframes bob {
    to {
      transform: translateY(-14px) rotate(8deg);
    }
  }
  .label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.75);
    padding: 5px 10px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.14);
  }
  .live-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 8px var(--ok);
  }
  h2 {
    font-size: 40px;
    line-height: 1.02;
    letter-spacing: -0.03em;
    margin: 16px 0 10px;
    font-weight: 700;
  }
  h3 {
    margin: 14px 0 8px;
    font-size: 19px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .together p {
    max-width: 380px;
  }
  .actions {
    display: flex;
    gap: 10px;
    margin-top: 20px;
  }
  .big-code {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 14px 0 4px;
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 44px;
    font-weight: 700;
    letter-spacing: 0.14em;
  }
  .people {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 12px;
    max-width: 420px;
  }
  .person {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 3px 12px 3px 3px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.1);
    font-weight: 500;
  }
  .paste-form {
    display: flex;
    gap: 8px;
    margin-top: 18px;
  }
  .block {
    margin-top: 38px;
  }
  .block-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }
  .scroller {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 168px;
    gap: 20px;
    overflow-x: auto;
    padding: 10px 10px 16px;
    margin: -10px -10px 0;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 22px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(168px, 1fr));
    gap: 26px 22px;
  }
  @container (max-width: 900px) {
    .hero {
      grid-template-columns: 1fr;
    }
    .orb {
      width: 120px;
      height: 120px;
      margin-top: -60px;
      opacity: 0.7;
    }
  }
  .sk-art {
    aspect-ratio: 1;
    border-radius: 18px;
  }
  .sk-line {
    height: 12px;
    border-radius: 6px;
    margin-top: 12px;
    width: 80%;
  }
  .sk-line.short {
    width: 50%;
    margin-top: 8px;
  }
  .err {
    padding: 24px;
    border-radius: var(--r);
    background: rgba(251, 113, 133, 0.08);
    border: 1px solid rgba(251, 113, 133, 0.25);
  }
  .err p {
    margin: 0 0 12px;
  }
</style>

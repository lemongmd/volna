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
  let copied = $state(false);

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

  function openLink(e: Event) {
    e.preventDefault();
    const v = link.trim();
    if (!v) return;
    ui.query = v;
    ui.view = 'search';
  }

  async function copyCode() {
    if (!room.code) return;
    await navigator.clipboard.writeText(room.code).catch(() => {});
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }

  $effect(() => {
    if (ui.ready && !items.length && !loading && !error) loadPick(pick);
  });
</script>

<header class="head">
  <h1 class="page-title">{hello}{profile.name ? `, ${profile.name}` : ''}</h1>
</header>

<section class="top">
  <div class="panel surface room-panel">
    {#if room.code}
      <div class="rp-main">
        <div class="eyebrow"><span class="live"></span> Комната</div>
        <div class="code-row">
          <span class="code mono">{room.code}</span>
          <button class="icon-btn" onclick={copyCode} title="Скопировать код" aria-label="Скопировать код">
            <Icon name={copied ? 'check' : 'copy'} size={16} />
          </button>
        </div>
        <div class="people">
          {#each room.members as m (m.uid)}
            <span class="person"><Avatar name={m.name} color={m.color} size={22} /> {m.name}</span>
          {/each}
        </div>
      </div>
      <div class="rp-actions">
        <button class="btn btn-primary" onclick={() => ((ui.panel = true), (ui.tab = 'chat'))}>
          <Icon name="chat" size={15} /> Чат
        </button>
        <button class="btn" onclick={() => ((ui.panel = true), (ui.tab = 'queue'))}>
          <Icon name="queue" size={15} /> Очередь
        </button>
      </div>
    {:else}
      <div class="rp-main">
        <div class="eyebrow"><Icon name="users" size={13} /> Слушать вместе</div>
        <h2>Одна музыка — одна секунда</h2>
        <p class="muted">Создай комнату и отправь код. Все услышат один и тот же трек синхронно, с общей очередью и чатом.</p>
      </div>
      <div class="rp-actions">
        <button class="btn btn-primary" onclick={() => (ui.modal = 'join')}>
          <Icon name="plus" size={15} /> Создать комнату
        </button>
        <button class="btn" onclick={() => (ui.modal = 'join')}>Войти по коду</button>
      </div>
    {/if}
  </div>

  <form class="panel surface link-panel" onsubmit={openLink}>
    <div class="eyebrow"><Icon name="link" size={13} /> По ссылке</div>
    <p class="muted">YouTube, SoundCloud, Spotify — трек, альбом или плейлист.</p>
    <div class="link-row">
      <input class="input" placeholder="Вставь ссылку…" bind:value={link} spellcheck="false" />
      <button class="btn btn-primary" type="submit" disabled={!isLink(link)} aria-label="Открыть">
        <Icon name="arrowRight" size={16} />
      </button>
    </div>
  </form>
</section>

{#if recent.length}
  <section class="block">
    <h2 class="section-title">Недавно слушали</h2>
    <div class="scroller">
      {#each recent.slice(0, 14) as t (t.id)}
        <TrackCard track={t} />
      {/each}
    </div>
  </section>
{/if}

<section class="block">
  <h2 class="section-title">Подборки</h2>
  <div class="chips">
    {#each PICKS as p, i}
      <button class="tab-chip" class:active={pick === i} onclick={() => loadPick(i)}>{p.label}</button>
    {/each}
  </div>
  {#if error}
    <div class="err surface">
      <p>Не получилось загрузить подборку: {error}</p>
      <button class="btn" onclick={() => loadPick(pick)}><Icon name="refresh" size={15} /> Ещё раз</button>
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
  .head {
    margin: 0 0 var(--s5);
  }
  .top {
    display: grid;
    grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
    gap: var(--s4);
  }
  .panel {
    padding: var(--s5);
  }
  .room-panel {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--s5);
  }
  .rp-main {
    min-width: 0;
  }
  .rp-actions {
    display: flex;
    gap: 8px;
    flex: none;
  }
  .eyebrow {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .live {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--live);
  }
  h2 {
    margin: var(--s3) 0 var(--s2);
    font-size: 20px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .room-panel p,
  .link-panel p {
    margin: 0;
    font-size: 13px;
    max-width: 440px;
  }
  .link-panel p {
    margin-top: var(--s3);
  }
  .code-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 6px 0 10px;
  }
  .code {
    font-size: 30px;
    font-weight: 700;
    letter-spacing: 0.12em;
    user-select: text;
  }
  .people {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .person {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 10px 2px 2px;
    border-radius: 99px;
    background: var(--panel-2);
    font-size: 13px;
  }
  .link-row {
    display: flex;
    gap: var(--s2);
    margin-top: var(--s4);
  }
  .link-row .btn {
    width: 36px;
    height: 36px;
    padding: 0;
    flex: none;
  }
  .block {
    margin-top: var(--s7);
  }
  .block .section-title {
    margin-bottom: var(--s4);
  }
  .scroller {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 176px;
    gap: var(--s5);
    overflow-x: auto;
    padding-bottom: var(--s3);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s2);
    margin-bottom: var(--s5);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(176px, 1fr));
    gap: 36px var(--s5);
  }
  .sk-art {
    aspect-ratio: 1;
    border-radius: 10px;
  }
  .sk-line {
    height: 11px;
    border-radius: 4px;
    margin-top: 10px;
    width: 80%;
  }
  .sk-line.short {
    width: 50%;
    margin-top: 6px;
  }
  .err {
    padding: var(--s5);
  }
  .err p {
    margin: 0 0 12px;
  }
  @container (max-width: 860px) {
    .top {
      grid-template-columns: 1fr;
    }
  }
  @container (max-width: 640px) {
    .room-panel {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>

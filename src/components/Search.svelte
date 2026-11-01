<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';
  import TrackRow from './TrackRow.svelte';
  import { settings, ui } from '../lib/state.svelte';
  import { api } from '../lib/api';
  import { searchCached } from '../lib/catalog';
  import { addToQueue, playNow } from '../lib/sync';
  import { errText, isLink } from '../lib/util';
  import type { Track } from '../lib/types';

  const SOURCES = [
    { id: 'ytm', label: 'YouTube Music', icon: 'music' },
    { id: 'yt', label: 'YouTube', icon: 'youtube' },
    { id: 'sc', label: 'SoundCloud', icon: 'cloud' },
  ] as const;

  let input: HTMLInputElement;
  let results = $state<Track[]>([]);
  let loading = $state(false);
  let error = $state('');
  let fromLink = $state(false);
  let lastRun = '';
  let seq = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function run(force = false) {
    const q = ui.query.trim();
    const key = `${settings.source}|${q}`;
    if (!q) {
      results = [];
      error = '';
      return;
    }
    if (!force && key === lastRun) return;
    lastRun = key;
    const my = ++seq;
    loading = true;
    error = '';
    try {
      const link = isLink(q);
      const res = link ? await api.resolveLink(q) : await searchCached(q, settings.source);
      if (my !== seq) return;
      fromLink = link;
      results = res;
      if (!res.length) error = 'Ничего не нашлось. Попробуй другой источник.';
    } catch (e) {
      if (my !== seq) return;
      results = [];
      error = errText(e);
    } finally {
      if (my === seq) loading = false;
    }
  }

  function onInput() {
    clearTimeout(timer);
    // ссылки не ищем «на лету» — только по Enter или после паузы подлиннее
    timer = setTimeout(() => run(), isLink(ui.query) ? 600 : 450);
  }

  function setSource(id: 'ytm' | 'yt' | 'sc') {
    settings.source = id;
    run(true);
  }

  function playAll() {
    if (!results.length) return;
    playNow(results[0]);
    if (results.length > 1) addToQueue(results.slice(1));
  }

  onMount(() => {
    input?.focus();
    if (ui.query) run(true);
  });
</script>

<div class="search-head">
  <form
    class="field glass liquid"
    onsubmit={(e) => {
      e.preventDefault();
      run(true);
    }}
  >
    <Icon name="search" size={20} />
    <input
      bind:this={input}
      bind:value={ui.query}
      oninput={onInput}
      placeholder="Трек, исполнитель или ссылка (YouTube, SoundCloud, Spotify…)"
      spellcheck="false"
    />
    {#if loading}<span class="spinner"></span>{/if}
    {#if ui.query}
      <button type="button" class="icon-btn" onclick={() => ((ui.query = ''), (results = []), (error = ''), input.focus())} aria-label="Очистить">
        <Icon name="x" size={16} />
      </button>
    {/if}
  </form>
  <div class="sources">
    {#each SOURCES as s}
      <button class="chip" class:active={settings.source === s.id} onclick={() => setSource(s.id)}>
        <span class="chip-in"><Icon name={s.icon} size={15} /> {s.label}</span>
      </button>
    {/each}
  </div>
</div>

{#if error}
  <div class="state">
    <Icon name="search" size={28} />
    <p>{error}</p>
  </div>
{:else if results.length}
  <div class="results-head">
    <h2 class="section-title">{fromLink ? 'По ссылке' : 'Результаты'}</h2>
    <span class="muted">{results.length} треков</span>
    <span class="grow"></span>
    {#if fromLink && results.length > 1}
      <button class="btn btn-glass" onclick={() => addToQueue(results)}><Icon name="plus" size={16} /> Всё в очередь</button>
    {/if}
    <button class="btn btn-primary" onclick={playAll}><Icon name="play" size={14} /> Слушать всё</button>
  </div>
  <div class="list glass" role="list">
    {#each results as t, i (t.id + i)}
      <TrackRow track={t} index={i} />
    {/each}
  </div>
{:else if loading}
  <div class="list glass">
    {#each Array(8) as _}
      <div class="sk-row"><div class="sk-c skeleton"></div><div class="sk-l skeleton"></div></div>
    {/each}
  </div>
{:else}
  <div class="state hint">
    <div class="hint-orb"><Icon name="wave" size={34} /></div>
    <h3>Найди что угодно</h3>
    <p class="muted">
      Ищем на YouTube Music, YouTube и SoundCloud — бесплатно и без цензуры. Совет: если трек «запикан», переключи источник
      на YouTube или SoundCloud — там часто лежат оригинальные версии.
    </p>
  </div>
{/if}

<style>
  .search-head {
    position: sticky;
    top: -12px;
    z-index: 5;
    padding: 12px 0 18px;
    margin-bottom: 6px;
  }
  .field {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 58px;
    padding: 0 12px 0 20px;
    border-radius: 999px;
    color: var(--muted);
  }
  .field input {
    flex: 1;
    min-width: 0;
    height: 100%;
    background: none;
    border: none;
    outline: none;
    font-size: 16px;
    color: var(--text);
    user-select: text;
  }
  .field input::placeholder {
    color: var(--faint);
  }
  .sources {
    display: flex;
    gap: 8px;
    margin-top: 14px;
  }
  .chip-in {
    display: inline-flex;
    align-items: center;
    gap: 7px;
  }
  .results-head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 10px 0 14px;
  }
  .grow {
    flex: 1;
  }
  .list {
    border-radius: var(--r-lg);
    padding: 8px;
  }
  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 10px;
    padding: 60px 20px;
    color: var(--muted);
  }
  .hint p {
    max-width: 520px;
    margin: 0;
  }
  .hint h3 {
    margin: 8px 0 0;
    font-size: 22px;
    color: var(--text);
  }
  .hint-orb {
    width: 84px;
    height: 84px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: #fff;
    background: radial-gradient(circle at 30% 25%, rgba(255, 255, 255, 0.5), transparent 30%),
      linear-gradient(135deg, #7c3aed, #c084fc 60%, #38bdf8);
    box-shadow: 0 20px 50px rgba(124, 58, 237, 0.5);
  }
  .sk-row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 12px 8px 54px;
  }
  .sk-c {
    width: 46px;
    height: 46px;
    border-radius: 10px;
  }
  .sk-l {
    height: 14px;
    width: 40%;
    border-radius: 7px;
  }
</style>

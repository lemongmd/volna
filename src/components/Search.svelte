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
    { id: 'ytm', label: 'YouTube Music' },
    { id: 'yt', label: 'YouTube' },
    { id: 'sc', label: 'SoundCloud' },
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

  function clear() {
    ui.query = '';
    results = [];
    error = '';
    input.focus();
  }

  onMount(() => {
    input?.focus();
    if (ui.query) run(true);
  });
</script>

<header class="head">
  <h1 class="page-title">Поиск</h1>
  <div class="bar">
    <div class="sources">
      {#each SOURCES as s}
        <button class="tab-chip" class:active={settings.source === s.id} onclick={() => setSource(s.id)}>{s.label}</button>
      {/each}
    </div>
    <form
      class="field"
      onsubmit={(e) => {
        e.preventDefault();
        run(true);
      }}
    >
      <Icon name="search" size={16} />
      <input
        bind:this={input}
        bind:value={ui.query}
        oninput={onInput}
        placeholder="Трек, исполнитель или ссылка"
        spellcheck="false"
      />
      {#if loading}
        <span class="spinner"></span>
      {:else if ui.query}
        <button type="button" class="clear" onclick={clear} aria-label="Очистить"><Icon name="x" size={14} /></button>
      {/if}
    </form>
  </div>
</header>

{#if error}
  <div class="state muted">{error}</div>
{:else if results.length}
  <div class="results-head">
    <span class="muted">{fromLink ? 'По ссылке' : 'Результаты'} · {results.length}</span>
    <span class="grow"></span>
    {#if fromLink && results.length > 1}
      <button class="btn" onclick={() => addToQueue(results)}><Icon name="plus" size={15} /> Всё в очередь</button>
    {/if}
    <button class="btn btn-primary" onclick={playAll}><Icon name="play" size={13} /> Слушать всё</button>
  </div>
  <div class="list surface" role="list">
    {#each results as t, i (t.id + i)}
      <TrackRow track={t} index={i} />
    {/each}
  </div>
{:else if loading}
  <div class="list surface">
    {#each Array(8) as _}
      <div class="sk-row"><div class="sk-c skeleton"></div><div class="sk-l skeleton"></div></div>
    {/each}
  </div>
{:else}
  <div class="state hint">
    <p>Ищем на YouTube Music, YouTube и SoundCloud — бесплатно и без цензуры.</p>
    <p class="muted">Если трек «запикан», переключи источник на YouTube или SoundCloud — там обычно лежит оригинал.</p>
  </div>
{/if}

<style>
  .head {
    margin: 8px 0 18px;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 14px;
  }
  .sources {
    display: flex;
    gap: 6px;
    flex: none;
  }
  .field {
    flex: 1;
    min-width: 200px;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 10px;
    border-radius: var(--r-sm);
    background: rgba(255, 255, 255, 0.07);
    color: var(--muted);
    border: 1px solid transparent;
    transition: border 0.15s, background 0.15s;
  }
  .field:focus-within {
    border-color: var(--stroke-2);
    background: rgba(255, 255, 255, 0.1);
  }
  .field input {
    flex: 1;
    min-width: 0;
    height: 100%;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    user-select: text;
  }
  .field input::placeholder {
    color: var(--faint);
  }
  .clear {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    color: var(--muted);
  }
  .clear:hover {
    background: var(--panel-2);
    color: var(--text);
  }
  .results-head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
  }
  .grow {
    flex: 1;
  }
  .list {
    padding: 6px;
  }
  .state {
    padding: 60px 20px;
    text-align: center;
  }
  .hint p {
    margin: 0 auto 6px;
    max-width: 480px;
  }
  .sk-row {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 52px;
    padding: 0 12px 0 48px;
  }
  .sk-c {
    width: 38px;
    height: 38px;
    border-radius: 6px;
  }
  .sk-l {
    height: 12px;
    width: 40%;
    border-radius: 4px;
  }
  @container (max-width: 760px) {
    .bar {
      flex-direction: column-reverse;
      align-items: stretch;
    }
  }
</style>

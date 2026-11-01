<script lang="ts">
  import { onDestroy, tick } from 'svelte';
  import Icon from './Icon.svelte';
  import { current, toast } from '../lib/state.svelte';
  import { currentTime } from '../lib/player';
  import { dispatch } from '../lib/sync';
  import { clearCustom, getLyrics, getOffset, setCustom, setOffset, type LyricsData } from '../lib/lyrics';
  import { errText } from '../lib/util';

  type Row = { t: number; text: string; dots?: boolean; end?: number };

  const cur = $derived(current());
  let data = $state<LyricsData | null>(null);
  let loading = $state(false);
  let error = $state('');
  let offset = $state(0);
  let active = $state(-1);
  let now = $state(0);
  let box: HTMLDivElement | undefined = $state();
  let fileInput: HTMLInputElement | undefined = $state();
  let userScrollUntil = 0;
  let loadedFor = '';
  let raf = 0;

  // Строки + «• • •» для вступления и длинных проигрышей
  const rows = $derived.by<Row[]>(() => {
    const lines = data?.lines;
    if (!lines?.length) return [];
    const out: Row[] = [];
    if (lines[0].t > 4) out.push({ t: 0, text: '', dots: true, end: lines[0].t });
    lines.forEach((l, i) => {
      const next = lines[i + 1];
      if (!l.text) {
        // пустая строка в LRC — пауза
        if (next && next.t - l.t > 4) out.push({ t: l.t, text: '', dots: true, end: next.t });
        return;
      }
      out.push({ t: l.t, text: l.text });
      if (next && next.t - l.t > 14) out.push({ t: l.t + 6, text: '', dots: true, end: next.t });
    });
    return out;
  });

  async function load(force = false) {
    const t = cur;
    if (!t) return;
    loadedFor = t.id;
    offset = getOffset(t);
    loading = true;
    error = '';
    data = null;
    active = -1;
    try {
      const d = await getLyrics(t, force);
      if (loadedFor !== t.id) return;
      data = d;
    } catch (e) {
      if (loadedFor === t.id) error = errText(e);
    } finally {
      if (loadedFor === t.id) loading = false;
    }
    await tick();
    if (box) box.scrollTop = 0;
  }

  $effect(() => {
    const id = cur?.id;
    if (id && id !== loadedFor) load();
  });

  function frame() {
    raf = requestAnimationFrame(frame);
    if (!rows.length) return;
    now = currentTime() + offset;
    let i = -1;
    for (let k = 0; k < rows.length; k++) {
      if (rows[k].t <= now) i = k;
      else break;
    }
    if (i !== active) {
      active = i;
      scrollToActive();
    }
  }
  raf = requestAnimationFrame(frame);
  onDestroy(() => cancelAnimationFrame(raf));

  function scrollToActive(smooth = true) {
    if (!box || Date.now() < userScrollUntil) return;
    const el = box.querySelector<HTMLElement>(`[data-i="${Math.max(0, active)}"]`);
    if (!el) return;
    const top = el.offsetTop - box.clientHeight * 0.32;
    box.scrollTo({ top: Math.max(0, top), behavior: smooth ? 'smooth' : 'auto' });
  }

  function onUserScroll() {
    userScrollUntil = Date.now() + 3500;
  }

  function seek(r: Row) {
    userScrollUntil = 0;
    dispatch({ t: 'seek', pos: Math.max(0, r.t - offset + 0.05) });
  }

  function shift(d: number) {
    if (!cur) return;
    offset = Math.round((offset + d) * 10) / 10;
    setOffset(cur, offset);
  }

  async function importFile(e: Event) {
    const f = (e.target as HTMLInputElement).files?.[0];
    if (!f || !cur) return;
    const text = await f.text();
    setCustom(cur, text);
    toast('Текст сохранён для этого трека');
    (e.target as HTMLInputElement).value = '';
    load();
  }

  function removeCustom() {
    if (!cur) return;
    clearCustom(cur);
    load(true);
  }

  const dotsProgress = (r: Row) => (r.end ? Math.min(1, Math.max(0, (now - r.t) / (r.end - r.t))) : 0);
</script>

<div class="lyrics">
  <div class="scroll" bind:this={box} onwheel={onUserScroll} ontouchmove={onUserScroll} role="region" aria-label="Текст песни">
    {#if loading}
      <div class="state"><span class="spinner"></span> Ищем текст…</div>
    {:else if rows.length}
      <div class="pad-top"></div>
      {#each rows as r, i (i)}
        {#if r.dots}
          <div class="dots" class:on={i === active} data-i={i} style="--p:{i === active ? dotsProgress(r) : 0}">
            <i></i><i></i><i></i>
          </div>
        {:else}
          <button
            class="line"
            class:on={i === active}
            class:past={i < active}
            data-i={i}
            style="--d:{active < 0 ? Math.min(i + 1, 5) : Math.min(Math.abs(i - active), 5)}"
            onclick={() => seek(r)}
          >
            {r.text}
          </button>
        {/if}
      {/each}
      <div class="pad-bottom"></div>
    {:else if data?.plain}
      <div class="plain">{data.plain}</div>
    {:else if data?.instrumental}
      <div class="state">Инструментал — без слов</div>
    {:else}
      <div class="state col">
        <Icon name="lyrics" size={28} />
        <p>{error ? `Не получилось найти текст: ${error}` : 'Для этого трека текста пока нет.'}</p>
        <p class="muted small">Можно загрузить свой .lrc-файл (текст с таймкодами) — он сохранится для этого трека.</p>
        <div class="row">
          <button class="btn btn-primary" onclick={() => fileInput?.click()}><Icon name="upload" size={14} /> Загрузить .lrc</button>
          <button class="btn" onclick={() => load(true)}><Icon name="refresh" size={14} /> Искать снова</button>
        </div>
      </div>
    {/if}
  </div>

  {#if data}
    <div class="toolbar">
      <span class="src">{data.source === 'custom' ? 'Свой текст' : 'LRCLIB'}</span>
      {#if data.lines}
        <span class="offset" title="Сдвиг текста относительно музыки">
          <button onclick={() => shift(-0.5)} aria-label="Текст раньше"><Icon name="dash" size={14} /></button>
          <span class="mono">{offset > 0 ? '+' : ''}{offset.toFixed(1)}с</span>
          <button onclick={() => shift(0.5)} aria-label="Текст позже"><Icon name="plusSm" size={14} /></button>
        </span>
      {/if}
      <span class="grow"></span>
      <button class="icon-btn" onclick={() => fileInput?.click()} title="Загрузить свой .lrc" aria-label="Загрузить свой .lrc">
        <Icon name="upload" size={15} />
      </button>
      {#if data.source === 'custom'}
        <button class="icon-btn" onclick={removeCustom} title="Убрать свой текст" aria-label="Убрать свой текст">
          <Icon name="trash" size={15} />
        </button>
      {/if}
    </div>
  {/if}
  <input bind:this={fileInput} type="file" accept=".lrc,.txt,text/plain" hidden onchange={importFile} />
</div>

<style>
  .lyrics {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 12px;
    mask-image: linear-gradient(180deg, transparent 0, #000 12%, #000 80%, transparent 100%);
    scrollbar-width: none;
  }
  .scroll::-webkit-scrollbar {
    display: none;
  }
  .pad-top {
    height: 28%;
  }
  .pad-bottom {
    height: 60%;
  }
  .line {
    display: block;
    width: 100%;
    text-align: left;
    padding: 10px 12px;
    margin: 2px 0;
    border-radius: 12px;
    font-size: 30px;
    font-weight: 750;
    line-height: 1.2;
    letter-spacing: -0.01em;
    color: #fff;
    opacity: 0.28;
    filter: blur(calc(var(--d) * 0.45px));
    transform: scale(0.97);
    transform-origin: left center;
    transition:
      opacity 0.45s ease,
      transform 0.45s cubic-bezier(0.2, 0.8, 0.2, 1),
      filter 0.45s ease;
    word-break: break-word;
  }
  .line.past {
    opacity: 0.22;
  }
  .line.on {
    opacity: 1;
    filter: none;
    transform: none;
  }
  .line:hover {
    opacity: 0.6;
    filter: none;
    background: rgba(255, 255, 255, 0.05);
  }
  .line.on:hover {
    opacity: 1;
  }
  .dots {
    display: flex;
    gap: 10px;
    padding: 18px 14px;
    opacity: 0.25;
    transition: opacity 0.4s;
  }
  .dots.on {
    opacity: 1;
  }
  .dots i {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #fff;
    opacity: 0.35;
  }
  .dots.on i {
    animation: breathe 2.4s ease-in-out infinite;
  }
  .dots.on i:nth-child(1) {
    opacity: calc(0.35 + min(1, var(--p) * 3) * 0.65);
  }
  .dots.on i:nth-child(2) {
    opacity: calc(0.35 + clamp(0, var(--p) * 3 - 1, 1) * 0.65);
    animation-delay: 0.2s;
  }
  .dots.on i:nth-child(3) {
    opacity: calc(0.35 + clamp(0, var(--p) * 3 - 2, 1) * 0.65);
    animation-delay: 0.4s;
  }
  @keyframes breathe {
    50% {
      transform: scale(1.25);
    }
  }
  .plain {
    white-space: pre-wrap;
    font-size: 20px;
    font-weight: 650;
    line-height: 1.5;
    padding: 24px 12px 80px;
    color: rgba(255, 255, 255, 0.85);
    user-select: text;
  }
  .state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 100%;
    color: var(--muted);
  }
  .state.col {
    flex-direction: column;
    text-align: center;
    padding: 0 20px;
  }
  .state p {
    margin: 0;
    max-width: 380px;
  }
  .small {
    font-size: 12.5px;
  }
  .row {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 4px 0;
    font-size: 12px;
    color: var(--faint);
  }
  .src {
    padding: 3px 8px;
    border-radius: 6px;
    background: var(--panel);
  }
  .offset {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    border-radius: 8px;
    background: var(--panel);
  }
  .offset button {
    width: 24px;
    height: 22px;
    display: grid;
    place-items: center;
    border-radius: 6px;
    color: var(--muted);
  }
  .offset button:hover {
    background: var(--panel-2);
    color: var(--text);
  }
  .offset .mono {
    min-width: 42px;
    text-align: center;
    color: var(--muted);
  }
  .grow {
    flex: 1;
  }
</style>

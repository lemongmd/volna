<script lang="ts">
  import { onDestroy, tick } from 'svelte';
  import Icon from './Icon.svelte';
  import { current, toast, ui } from '../lib/state.svelte';
  import { currentTime } from '../lib/player';
  import { dispatch } from '../lib/sync';
  import {
    acceptCandidate,
    clearCustom,
    clearPicked,
    getCandidates,
    getLyrics,
    getOffset,
    previewLines,
    setCustom,
    setOffset,
    type Candidate,
    type LyricsData,
  } from '../lib/lyrics';
  import { errText, fmtTime } from '../lib/util';

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
  // «Оно?»: варианты по одному названию и текущий показанный
  let cands = $state<Candidate[]>([]);
  let ci = $state(0);
  let exhausted = $state(false);
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

  async function load(force = false, skipExact = false) {
    const t = cur;
    if (!t) return;
    loadedFor = t.id;
    offset = getOffset(t);
    loading = true;
    error = '';
    data = null;
    active = -1;
    cands = [];
    ci = 0;
    exhausted = false;
    try {
      const d = skipExact ? null : await getLyrics(t, force);
      if (loadedFor !== t.id) return;
      data = d;
      if (!d) {
        // Точный поиск не помог — ищем по одному названию и спрашиваем «Оно?»
        const list = await getCandidates(t);
        if (loadedFor !== t.id) return;
        const sure = list.length === 1 && t.duration > 0 && Math.abs(list[0].duration - t.duration) <= 3;
        if (sure) data = acceptCandidate(t, list[0]);
        else cands = list;
      }
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
    // пока листаем совпадения, текст не должен уезжать за играющей строкой
    if (!box || (finding && hits.length) || Date.now() < userScrollUntil) return;
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

  function yes() {
    if (!cur || !cands[ci]) return;
    data = acceptCandidate(cur, cands[ci]);
    cands = [];
    toast('Запомнили текст для этого трека');
  }

  function no() {
    if (ci < cands.length - 1) ci += 1;
    else {
      cands = [];
      exhausted = true;
    }
  }

  function chooseAnother() {
    if (!cur) return;
    clearPicked(cur);
    load(false, true);
  }

  // ───── поиск по строчкам (Ctrl+F → «Строчки») ─────
  let finding = $state(false);
  let fq = $state('');
  let fsel = $state(0);
  let fInput: HTMLInputElement | undefined = $state();
  let hitsEl: HTMLDivElement | undefined = $state();

  /** Без регистра, ё = е, без знаков препинания — «dont» найдёт «don't». */
  const normalize = (x: string) =>
    x
      .toLowerCase()
      .replace(/ё/g, 'е')
      .replace(/['’`]/g, '')
      .replace(/[^\p{L}\p{N}]+/gu, ' ')
      .trim();

  const words = $derived(normalize(fq).split(' ').filter(Boolean));
  const hits = $derived.by(() => {
    if (!finding || !words.length) return [] as { r: Row; i: number }[];
    const out: { r: Row; i: number }[] = [];
    rows.forEach((r, i) => {
      if (r.dots) return;
      const n = normalize(r.text);
      if (words.every((w) => n.includes(w))) out.push({ r, i });
    });
    return out;
  });
  const hitSet = $derived(new Set(hits.map((h) => h.i)));
  const pickRow = $derived(hits[fsel]?.i ?? -1);
  const hlRe = $derived.by(() => {
    if (!finding || !words.length) return null;
    const src = words.map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/е/g, '[её]')).join('|');
    return new RegExp(`(${src})`, 'giu');
  });

  function hl(text: string): { s: string; m: boolean }[] {
    if (!hlRe) return [{ s: text, m: false }];
    return text
      .split(hlRe)
      .map((x, i) => ({ s: x, m: i % 2 === 1 }))
      .filter((p) => p.s);
  }

  async function openFind() {
    finding = true;
    await tick();
    fInput?.focus();
    fInput?.select();
  }

  function closeFind() {
    finding = false;
    fq = '';
    fsel = 0;
    userScrollUntil = 0;
    scrollToActive();
  }

  function stepHit(d: number) {
    if (!hits.length) return;
    fsel = (fsel + d + hits.length) % hits.length;
  }

  function goHit(i = fsel) {
    const h = hits[i];
    if (!h) return;
    seek(h.r);
    closeFind();
  }

  function onFindKey(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      stepHit(e.key === 'ArrowDown' ? 1 : -1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      goHit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      closeFind();
    }
  }

  // Запрос от Ctrl+F: открыть строку поиска (или вернуть в неё фокус)
  $effect(() => {
    if (!ui.lineFindReq) return;
    ui.lineFindReq = false;
    openFind();
  });

  // Выбранное совпадение показываем в тексте и в списке, но воспроизведение не трогаем
  $effect(() => {
    const row = pickRow;
    const j = fsel;
    if (!finding || row < 0) return;
    tick().then(() => {
      const el = box?.querySelector<HTMLElement>(`[data-i="${row}"]`);
      if (box && el) box.scrollTo({ top: Math.max(0, el.offsetTop - box.clientHeight * 0.32), behavior: 'smooth' });
      hitsEl?.querySelector<HTMLElement>(`[data-h="${j}"]`)?.scrollIntoView({ block: 'nearest' });
    });
  });

  const dotsProgress = (r: Row) => (r.end ? Math.min(1, Math.max(0, (now - r.t) / (r.end - r.t))) : 0);
</script>

<div class="lyrics" class:q={finding && hits.length > 0}>
  {#if finding}
    <div class="finder" role="search">
      <div class="ffield">
        <Icon name="search" size={16} />
        <input
          bind:this={fInput}
          bind:value={fq}
          oninput={() => (fsel = 0)}
          onkeydown={onFindKey}
          placeholder="Найти строчку в тексте"
          spellcheck="false"
          aria-label="Поиск по строчкам"
        />
        {#if fq.trim()}
          <span class="fcount mono">{hits.length ? `${fsel + 1} / ${hits.length}` : 'Нет совпадений'}</span>
        {/if}
        <button class="fbtn" onclick={() => stepHit(-1)} disabled={hits.length < 2} aria-label="Предыдущее совпадение"><Icon name="chevUp" size={14} /></button>
        <button class="fbtn" onclick={() => stepHit(1)} disabled={hits.length < 2} aria-label="Следующее совпадение"><Icon name="chevDown" size={14} /></button>
        <button class="fbtn" onclick={closeFind} aria-label="Закрыть поиск"><Icon name="x" size={14} /></button>
      </div>
      {#if !rows.length && !loading}
        <div class="fnote">Поиск работает по тексту с таймкодами — у этого трека его пока нет.</div>
      {:else if hits.length}
        <div class="hits" bind:this={hitsEl} role="listbox" aria-label="Найденные строчки">
          {#each hits as h, j (h.i)}
            <button
              class="hit-row"
              class:sel={j === fsel}
              role="option"
              aria-selected={j === fsel}
              data-h={j}
              onmousemove={() => fsel !== j && (fsel = j)}
              onclick={() => goHit(j)}
            >
              <span class="ht mono">{fmtTime(h.r.t)}</span>
              <span class="hx">{#each hl(h.r.text) as p}{#if p.m}<mark>{p.s}</mark>{:else}{p.s}{/if}{/each}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
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
            class:hit={hitSet.has(i)}
            class:pick={i === pickRow}
            data-i={i}
            style="--d:{active < 0 ? Math.min(i + 1, 5) : Math.min(Math.abs(i - active), 5)}"
            onclick={() => seek(r)}
          >
            {#each hl(r.text) as p}{#if p.m}<mark>{p.s}</mark>{:else}{p.s}{/if}{/each}
          </button>
        {/if}
      {/each}
      <div class="pad-bottom"></div>
    {:else if cands.length}
      {@const c = cands[ci]}
      <div class="ask">
        <div class="ask-head">
          Точного совпадения нет — нашли по названию
          {#if cands.length > 1}<span class="ask-count">вариант {ci + 1} из {cands.length}</span>{/if}
        </div>
        <div class="ask-meta">
          <span class="ask-title">{c.title}</span>
          <span class="muted">{c.artist}{c.album ? ` · ${c.album}` : ''}{c.duration ? ` · ${fmtTime(c.duration)}` : ''}</span>
        </div>
        <div class="ask-lines">
          {#each previewLines(c) as l}
            <p>{l}</p>
          {/each}
          <p class="ask-more">…</p>
        </div>
        <div class="ask-q">Оно?</div>
        <div class="row">
          <button class="btn btn-primary btn-lg" onclick={yes}><Icon name="check" size={15} /> Да</button>
          <button class="btn btn-lg" onclick={no}>{ci < cands.length - 1 ? 'Нет, следующий' : 'Нет'}</button>
        </div>
      </div>
    {:else if data?.plain}
      <div class="plain">{data.plain}</div>
    {:else if data?.instrumental}
      <div class="state">Инструментал — без слов</div>
    {:else}
      <div class="state col">
        <Icon name="lyrics" size={28} />
        <p>{error || (exhausted ? 'Подходящий текст не нашёлся.' : 'Для этого трека текста пока нет.')}</p>
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
      <span class="src">
        {data.source === 'custom' ? 'Свой текст' : data.source === 'ytm' ? 'YouTube Music' : 'LRCLIB'}{data.picked
          ? ' · выбран вручную'
          : ''}
      </span>
      {#if data.lines}
        <span class="offset" title="Сдвиг текста относительно музыки">
          <button class="tb-btn" onclick={() => shift(-0.5)} aria-label="Текст раньше на 0,5 с" title="Текст раньше">
            <Icon name="dash" size={18} />
          </button>
          <span class="mono">{offset > 0 ? '+' : ''}{offset.toFixed(1)} с</span>
          <button class="tb-btn" onclick={() => shift(0.5)} aria-label="Текст позже на 0,5 с" title="Текст позже">
            <Icon name="plusSm" size={18} />
          </button>
        </span>
      {/if}
      <span class="grow"></span>
      <button class="tb-btn" onclick={openFind} title="Найти строчку (Ctrl+F)" aria-label="Найти строчку в тексте">
        <Icon name="search" size={18} />
      </button>
      <button class="tb-btn" onclick={() => fileInput?.click()} title="Загрузить свой .lrc" aria-label="Загрузить свой .lrc">
        <Icon name="upload" size={18} />
      </button>
      {#if data.source !== 'custom'}
        <button class="tb-btn" onclick={chooseAnother} title="Не тот текст — выбрать другой" aria-label="Выбрать другой текст">
          <Icon name="refresh" size={18} />
        </button>
      {/if}
      {#if data.source === 'custom'}
        <button class="tb-btn" onclick={removeCustom} title="Убрать свой текст" aria-label="Убрать свой текст">
          <Icon name="trash" size={18} />
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
    padding: 0 var(--s2);
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
    max-width: 600px;
    /* ровные переносы: без одинокого короткого слова на второй строке */
    text-wrap: balance;
    text-align: left;
    padding: var(--s2) var(--s2);
    margin: 0;
    border-radius: var(--r-sm);
    font-size: 36px;
    font-family: var(--f-head);
    font-weight: 900;
    font-optical-sizing: auto;
    font-variation-settings: var(--head-axes);
    line-height: 1.2;
    letter-spacing: -0.01em;
    color: #fff;
    opacity: 0.28;
    /* лёгкий расфокус дальних строк: читаются, но фокус остаётся на активной */
    filter: blur(calc(var(--d) * 0.22px));
    /* неактивные строки меньше активной; через transform — без скачков вёрстки */
    transform: scale(0.84);
    transform-origin: left center;
    transition:
      opacity 0.45s ease,
      transform 0.45s cubic-bezier(0.2, 0.8, 0.2, 1),
      filter 0.45s ease;
    overflow-wrap: break-word;
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
  /* поиск по строчкам */
  .finder {
    position: absolute;
    z-index: 5;
    top: 0;
    left: 0;
    right: 0;
    max-width: 620px;
    animation: fade-in 0.16s ease both;
  }
  .ffield {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 44px;
    padding: 0 var(--s2) 0 var(--s4);
    border-radius: var(--pill);
    color: var(--muted);
    background: rgba(30, 30, 37, 0.92);
    border: 1px solid var(--stroke-2);
    backdrop-filter: var(--blur);
    -webkit-backdrop-filter: var(--blur);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  }
  .ffield input {
    flex: 1;
    min-width: 0;
    height: 100%;
    margin-left: 2px;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    user-select: text;
  }
  .ffield input::placeholder {
    color: var(--faint);
  }
  .fcount {
    flex: none;
    font-size: 12.5px;
    padding: 0 var(--s2);
    white-space: nowrap;
  }
  .fbtn {
    display: grid;
    place-items: center;
    flex: none;
    width: 30px;
    height: 30px;
    border-radius: var(--pill);
    color: var(--muted);
    transition:
      background 0.15s,
      color 0.15s;
  }
  .fbtn:hover:not(:disabled) {
    background: var(--panel-3);
    color: var(--text);
  }
  .fbtn:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .hits,
  .fnote {
    margin-top: 6px;
    border-radius: var(--r-lg);
    background: rgba(30, 30, 37, 0.94);
    border: 1px solid var(--stroke-2);
    backdrop-filter: var(--blur);
    -webkit-backdrop-filter: var(--blur);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.45);
  }
  .hits {
    max-height: min(42vh, 360px);
    overflow-y: auto;
    padding: 6px;
  }
  .fnote {
    padding: var(--s3) var(--s4);
    font-size: 13px;
    color: var(--muted);
  }
  .hit-row {
    display: flex;
    align-items: baseline;
    gap: var(--s3);
    width: 100%;
    padding: 8px var(--s3);
    border-radius: var(--r-sm);
    text-align: left;
    color: var(--muted);
  }
  .hit-row.sel {
    background: var(--panel-2);
    color: var(--text);
  }
  .ht {
    flex: none;
    min-width: 36px;
    font-size: 12px;
    color: var(--faint);
  }
  .hx {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  mark {
    background: rgba(255, 255, 255, 0.24);
    color: inherit;
    border-radius: 4px;
    padding: 0 2px;
    margin: 0 -2px;
  }
  .lyrics.q .line:not(.hit) {
    opacity: 0.12;
    filter: none;
  }
  .lyrics.q .line.hit {
    opacity: 0.7;
    filter: none;
    transform: scale(0.9);
  }
  .lyrics.q .line.pick {
    opacity: 1;
    transform: none;
    background: rgba(255, 255, 255, 0.07);
  }
  .dots {
    display: flex;
    gap: var(--s2);
    padding: var(--s2) var(--s2);
    min-height: 48px;
    align-items: center;
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
    padding: var(--s5) var(--s2) 80px;
    color: rgba(255, 255, 255, 0.85);
    user-select: text;
  }
  .state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s2);
    height: 100%;
    color: var(--muted);
  }
  .state.col {
    flex-direction: column;
    text-align: center;
    padding: 0 var(--s5);
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
    margin-top: var(--s4);
  }
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s2);
    padding-top: var(--s2);
    font-size: 13px;
    color: var(--muted);
  }
  .src {
    display: inline-flex;
    align-items: center;
    height: 40px;
    padding: 0 var(--s4);
    border-radius: var(--pill);
    background: var(--panel-2);
    color: var(--muted);
    white-space: nowrap;
  }
  .offset {
    display: inline-flex;
    align-items: center;
    border-radius: var(--pill);
    background: var(--panel-2);
  }
  .offset .mono {
    min-width: 48px;
    text-align: center;
    color: var(--text);
    font-weight: 600;
  }
  /* кнопки панели: область нажатия 40×40, контрастные иконки */
  .tb-btn {
    width: 40px;
    height: 40px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: var(--pill);
    color: var(--text);
    transition: background 0.15s;
  }
  .tb-btn:hover {
    background: var(--panel-3);
  }
  .toolbar > .tb-btn {
    background: var(--panel-2);
  }
  .toolbar > .tb-btn:hover {
    background: var(--panel-3);
  }
  .grow {
    flex: 1;
  }
  .ask {
    max-width: 560px;
    padding: var(--s6) var(--s2) var(--s5);
    animation: fade-in 0.25s ease both;
  }
  .ask-head {
    display: flex;
    align-items: center;
    gap: var(--s2);
    font-size: 13px;
    color: var(--muted);
  }
  .ask-count {
    padding: 0 var(--s2);
    line-height: 24px;
    border-radius: var(--r-sm);
    background: var(--panel-2);
    font-size: 12px;
  }
  .ask-meta {
    display: flex;
    flex-direction: column;
    gap: 0;
    margin-top: var(--s4);
  }
  .ask-title {
    font-size: 17px;
    font-weight: 650;
  }
  .ask-lines {
    margin-top: var(--s5);
    padding-left: var(--s4);
    border-left: 2px solid var(--stroke-2);
  }
  .ask-lines p {
    margin: 0 0 var(--s2);
    font-size: 22px;
    font-weight: 700;
    line-height: 1.3;
    color: rgba(255, 255, 255, 0.9);
  }
  .ask-lines .ask-more {
    color: var(--faint);
  }
  .ask-q {
    margin-top: var(--s5);
    font-family: var(--f-head);
    font-variation-settings: var(--head-axes);
    font-size: 26px;
    font-weight: 750;
  }
  .ask .row {
    margin-top: var(--s4);
  }
  /* узкое окно / телефон: шрифт меньше, чтобы в строке помещались слова */
  @container (max-width: 720px) {
    .line {
      font-size: 26px;
    }
    .ask-lines p {
      font-size: 18px;
    }
    .ask-q {
      font-size: 22px;
    }
  }
</style>

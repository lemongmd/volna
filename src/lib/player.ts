/**
 * Плеер: один <audio>, который следует общему состоянию (ps).
 * Хост — эталон: когда его трек реально заиграл, он «ставит якорь».
 * Участники подстраиваются под якорь: перемоткой при большом расхождении
 * и лёгким изменением скорости при маленьком (на слух незаметно).
 */
import { api } from './api';
import { current, isHost, pl, posAt, ps, room, roomNow, toast } from './state.svelte';
import { dispatch } from './sync';
import type { QItem } from './types';
import { errText } from './util';

const audio = new Audio();
audio.preload = 'auto';

let loadedQid: string | null = null;
let loadingQid: string | null = null;
let failedQid: string | null = null;
let retriedQid: string | null = null;
let wasWaiting = false;
let msKey = '';

export function init() {
  audio.addEventListener('timeupdate', () => {
    if (loadedQid && loadedQid === current()?.qid) pl.pos = audio.currentTime;
  });
  audio.addEventListener('durationchange', () => {
    if (isFinite(audio.duration) && audio.duration > 0) pl.dur = audio.duration;
  });
  audio.addEventListener('waiting', () => {
    pl.buffering = true;
    wasWaiting = true;
  });
  audio.addEventListener('playing', onPlaying);
  audio.addEventListener('pause', () => (pl.buffering = false));
  audio.addEventListener('ended', () => {
    const c = current();
    if (c && loadedQid === c.qid && isHost()) dispatch({ t: 'ended', qid: c.qid });
  });
  audio.addEventListener('error', onError);
  setInterval(tick, 1000);
  setupMediaSession();
  // отладка: позиция плеера (используется в автотестах синхронизации)
  (window as unknown as { __volnaPos: () => number }).__volnaPos = () => +audio.currentTime.toFixed(2);
}

export function setVolume(v: number, muted: boolean) {
  // квадратичная кривая — громкость меняется «на слух» равномерно
  audio.volume = muted ? 0 : Math.max(0, Math.min(1, v * v));
}

/** Текущая позиция загруженного трека (для хоста), иначе null. */
export function positionFor(qid: string | undefined): number | null {
  if (!qid || loadedQid !== qid || loadingQid === qid) return null;
  return audio.currentTime;
}

export function sync() {
  updateMediaSession();
  const c = current();
  if (!c) {
    if (loadedQid) {
      audio.pause();
      audio.removeAttribute('src');
      audio.load();
      loadedQid = null;
    }
    loadingQid = null;
    pl.loading = false;
    pl.pos = 0;
    pl.dur = 0;
    return;
  }
  if (c.qid === loadedQid) {
    align();
    return;
  }
  if (c.qid === loadingQid || c.qid === failedQid) return;
  void load(c);
}

function waitMeta(): Promise<void> {
  return new Promise((resolve, reject) => {
    if (audio.readyState >= 1) return resolve();
    const done = (fn: () => void) => {
      clearTimeout(timer);
      audio.removeEventListener('loadedmetadata', ok);
      audio.removeEventListener('error', bad);
      fn();
    };
    const ok = () => done(resolve);
    const bad = () => done(() => reject(new Error('Не удалось открыть поток')));
    const timer = setTimeout(() => done(() => reject(new Error('Поток не отвечает'))), 30000);
    audio.addEventListener('loadedmetadata', ok);
    audio.addEventListener('error', bad);
  });
}

async function load(c: QItem, force = false) {
  loadingQid = c.qid;
  pl.loading = true;
  pl.buffering = false;
  pl.error = '';
  pl.dur = c.duration;
  pl.pos = 0;
  audio.pause();
  try {
    const r = await api.resolve(c.url, force);
    if (loadingQid !== c.qid) return;
    if (r.duration && !c.duration) pl.dur = r.duration;
    loadedQid = c.qid;
    audio.src = api.streamUrl(r.key);
    await waitMeta();
    if (loadingQid !== c.qid) return;
    loadingQid = null;
    pl.loading = false;
    align();
    prefetch();
  } catch (e) {
    if (loadingQid !== c.qid) return;
    loadingQid = null;
    pl.loading = false;
    failedQid = c.qid;
    pl.error = errText(e);
    toast(`Не удалось загрузить «${c.title}»: ${pl.error}`, 'error');
    // хост пропускает недоступный трек, чтобы комната не встала
    if (isHost() && ps.playing) {
      setTimeout(() => {
        if (current()?.qid === c.qid) dispatch({ t: 'ended', qid: c.qid });
      }, 2000);
    }
  }
}

/** Заранее достаём ссылку на следующий трек — переключение будет мгновенным. */
function prefetch() {
  const next = ps.queue[ps.index + 1];
  if (next) api.resolve(next.url).catch(() => {});
}

function onError() {
  const c = current();
  if (!c || loadedQid !== c.qid || loadingQid === c.qid) return;
  // Ссылка могла протухнуть — одна попытка достать свежую.
  if (retriedQid !== c.qid) {
    retriedQid = c.qid;
    loadedQid = null;
    void load(c, true);
    return;
  }
  failedQid = c.qid;
  loadedQid = null;
  toast(`Ошибка воспроизведения «${c.title}»`, 'error');
  if (isHost() && ps.playing) dispatch({ t: 'ended', qid: c.qid });
}

function onPlaying() {
  pl.buffering = false;
  const c = current();
  if (!c || loadedQid !== c.qid) return;
  // Хост реально заиграл (или вышел из буферизации) — ставим якорь для всех.
  if (isHost() && ps.playing && (!ps.ready || wasWaiting)) {
    dispatch({ t: 'ready', qid: c.qid, pos: audio.currentTime });
  }
  wasWaiting = false;
}

function seekTo(t: number) {
  try {
    audio.currentTime = Math.max(0, t);
  } catch {
    /* метаданные ещё не загружены */
  }
}

function play() {
  if (!audio.paused) return;
  audio.play().catch((e) => {
    if (e?.name !== 'AbortError') console.warn('play()', e);
  });
}

/** Привести <audio> к общему состоянию. */
export function align() {
  const c = current();
  if (!c || loadedQid !== c.qid || loadingQid === c.qid) return;

  if (!ps.playing) {
    audio.playbackRate = 1;
    if (!audio.paused) audio.pause();
    const t = Math.max(0, ps.anchorPos);
    if (Math.abs(audio.currentTime - t) > 0.3) seekTo(t);
    pl.pos = t;
    return;
  }

  if (isHost()) {
    audio.playbackRate = 1;
    if (!ps.ready) {
      if (Math.abs(audio.currentTime - ps.anchorPos) > 0.3) seekTo(ps.anchorPos);
    } else {
      const target = posAt(ps, roomNow());
      if (Math.abs(audio.currentTime - target) > 0.6) seekTo(target);
    }
    play();
    return;
  }

  // участник: ждём, пока хост начнёт играть
  if (!ps.ready) {
    if (!audio.paused) audio.pause();
    if (audio.currentTime > 0.05) seekTo(ps.anchorPos);
    return;
  }
  const target = posAt(ps, roomNow());
  if (c.duration && target > c.duration + 1) {
    audio.pause();
    return;
  }
  if (Math.abs(audio.currentTime - target) > 0.4) seekTo(target + 0.1);
  play();
}

function tick() {
  const c = current();
  if (!c) return;
  if (loadedQid !== c.qid || loadingQid === c.qid) {
    pl.pos = Math.max(0, posAt(ps, roomNow()));
    return;
  }
  if (isHost()) {
    // страховка: если событие ended потерялось
    if (ps.playing && ps.ready && audio.ended) dispatch({ t: 'ended', qid: c.qid });
    else if (ps.playing && audio.paused && !pl.buffering) play();
    return;
  }
  if (!ps.playing || !ps.ready) return;
  if (audio.paused) {
    align();
    return;
  }
  if (audio.seeking || pl.buffering) {
    audio.playbackRate = 1;
    return;
  }
  const target = posAt(ps, roomNow());
  const diff = audio.currentTime - target;
  if (Math.abs(diff) > 0.4) {
    audio.playbackRate = 1;
    seekTo(target + 0.1);
  } else if (Math.abs(diff) > 0.04) {
    // впереди — чуть медленнее, отстаём — чуть быстрее
    audio.playbackRate = 1 - Math.max(-0.05, Math.min(0.05, diff * 0.4));
  } else {
    audio.playbackRate = 1;
  }
}

// ─────────── медиаклавиши и плашка Windows (SMTC) ───────────

function setupMediaSession() {
  const ms = navigator.mediaSession;
  if (!ms) return;
  const h = (a: MediaSessionAction, fn: MediaSessionActionHandler) => {
    try {
      ms.setActionHandler(a, fn);
    } catch {
      /* не поддерживается */
    }
  };
  h('play', () => dispatch({ t: 'play' }));
  h('pause', () => dispatch({ t: 'pause' }));
  h('nexttrack', () => dispatch({ t: 'next' }));
  h('previoustrack', () => dispatch({ t: 'prev' }));
  h('seekto', (d) => d.seekTime != null && dispatch({ t: 'seek', pos: d.seekTime }));
}

function updateMediaSession() {
  const ms = navigator.mediaSession;
  if (!ms) return;
  const c = current();
  const key = (c?.qid ?? '') + (room.code ?? '');
  if (key !== msKey) {
    msKey = key;
    ms.metadata = c
      ? new MediaMetadata({
          title: c.title,
          artist: c.artist,
          album: room.code ? `Волна · комната ${room.code}` : 'Волна',
          artwork: c.thumb ? [{ src: c.thumb, sizes: '512x512' }] : [],
        })
      : null;
  }
  ms.playbackState = c ? (ps.playing ? 'playing' : 'paused') : 'none';
}


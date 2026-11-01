/**
 * Тексты песен: синхронизированные (LRC) из LRCLIB или свой .lrc-файл.
 * Свой файл хранится локально и всегда важнее найденного — так можно
 * подставить оригинальный текст, если в базе лежит «запиканная» версия.
 */
import { api } from './api';
import type { Track } from './types';
import { load, save } from './util';

export interface Line {
  t: number;
  text: string;
}

export interface LyricsData {
  lines: Line[] | null;
  plain: string | null;
  instrumental: boolean;
  source: 'lrclib' | 'custom';
}

const cache = new Map<string, LyricsData | null>();
const pending = new Map<string, Promise<LyricsData | null>>();

/** Разбирает LRC: `[01:23.45] строка`, несколько меток на строку, `[offset:+250]`. */
export function parseLRC(text: string): Line[] {
  const out: Line[] = [];
  let offset = 0;
  for (const raw of text.split(/\r?\n/)) {
    const off = raw.match(/^\[offset:\s*([+-]?\d+)\s*\]/i);
    if (off) {
      offset = parseInt(off[1], 10) / 1000;
      continue;
    }
    const tags = [...raw.matchAll(/\[(\d{1,3}):(\d{1,2}(?:[.:]\d{1,3})?)\]/g)];
    if (!tags.length) continue;
    const content = raw.replace(/\[[^\]]*\]/g, '').replace(/<\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?>/g, '').trim();
    for (const m of tags) {
      const t = parseInt(m[1], 10) * 60 + parseFloat(m[2].replace(':', '.'));
      out.push({ t: Math.max(0, t - offset), text: content });
    }
  }
  return out.sort((a, b) => a.t - b.t);
}

function fromText(text: string, source: LyricsData['source']): LyricsData {
  const lines = parseLRC(text);
  if (lines.length) return { lines, plain: null, instrumental: false, source };
  return { lines: null, plain: text.trim(), instrumental: false, source };
}

function customKey(t: Track) {
  return 'lyrics:' + t.id;
}

export function getCustom(t: Track): LyricsData | null {
  const text = load<string | null>(customKey(t), null);
  return text ? fromText(text, 'custom') : null;
}

export function setCustom(t: Track, text: string) {
  save(customKey(t), text);
}

export function clearCustom(t: Track) {
  try {
    localStorage.removeItem('volna:' + customKey(t));
  } catch {
    /* ничего */
  }
}

export function getOffset(t: Track): number {
  return load<number>('lyroff:' + t.id, 0);
}

export function setOffset(t: Track, v: number) {
  save('lyroff:' + t.id, Math.round(v * 10) / 10);
}

export async function getLyrics(t: Track, force = false): Promise<LyricsData | null> {
  const custom = getCustom(t);
  if (custom) return custom;
  if (!force && cache.has(t.id)) return cache.get(t.id)!;
  if (!force && pending.has(t.id)) return pending.get(t.id)!;
  const p = api
    .lyrics(t.title, t.artist, t.duration)
    .then((r) => {
      let data: LyricsData | null = null;
      if (r?.synced) data = fromText(r.synced, 'lrclib');
      else if (r?.plain) data = { lines: null, plain: r.plain.trim(), instrumental: false, source: 'lrclib' };
      else if (r?.instrumental) data = { lines: null, plain: null, instrumental: true, source: 'lrclib' };
      cache.set(t.id, data);
      return data;
    })
    .finally(() => pending.delete(t.id));
  pending.set(t.id, p);
  return p;
}

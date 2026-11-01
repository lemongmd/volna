import { api } from './api';
import type { Track } from './types';

// Небольшой кэш поиска на сессию — повторные запросы мгновенные.
const cache = new Map<string, { at: number; items: Track[] }>();
const TTL = 15 * 60 * 1000;

export async function searchCached(query: string, source: string): Promise<Track[]> {
  const key = `${source}|${query.trim().toLowerCase()}`;
  const hit = cache.get(key);
  if (hit && Date.now() - hit.at < TTL) return hit.items;
  const items = await api.search(query, source);
  cache.set(key, { at: Date.now(), items });
  return items;
}

export const PICKS: { label: string; query: string; emoji: string }[] = [
  { label: 'Русский рэп', query: 'русский рэп хиты', emoji: '🎤' },
  { label: 'Фонк', query: 'phonk', emoji: '🚗' },
  { label: 'Новинки', query: 'новые треки 2026', emoji: '✨' },
  { label: 'Lo-fi', query: 'lofi hip hop', emoji: '🌙' },
  { label: 'Поп', query: 'поп хиты', emoji: '💜' },
  { label: 'Рок', query: 'русский рок', emoji: '🎸' },
  { label: 'Электроника', query: 'electronic house hits', emoji: '🎧' },
  { label: 'Нулевые', query: 'хиты 2000-х', emoji: '📼' },
];

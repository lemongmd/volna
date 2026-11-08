import type { ChatMsg, Member, PlayState, QItem, Track, View } from './types';
import { load, pickColor, rid, save } from './util';

export interface Profile {
  uid: string;
  name: string;
  color: string;
}

export interface Settings {
  volume: number;
  muted: boolean;
  broker: string;
  fx: boolean;
  cookieBrowser: 'none' | 'chrome' | 'firefox' | 'edge' | 'brave';
  source: 'ytm' | 'yt' | 'sc';
}

export const profile: Profile = $state(load<Profile>('profile', { uid: rid(12), name: '', color: pickColor() }));

export const settings: Settings = $state(
  load<Settings>('settings', { volume: 0.8, muted: false, broker: '', fx: true, cookieBrowser: 'none', source: 'ytm' }),
);

export const ui = $state({
  view: 'home' as View,
  panel: false,
  tab: 'queue' as 'queue' | 'chat' | 'people',
  npTab: 'stage' as 'stage' | 'lyrics' | 'next' | 'room',
  modal: null as null | 'join' | 'settings' | 'profile',
  ready: false,
  query: '',
});

export const ps: PlayState = $state({
  queue: [],
  index: -1,
  playing: false,
  ready: false,
  anchorPos: 0,
  anchorTime: 0,
  hostId: profile.uid,
  hostOnly: false,
  repeat: 'off',
  rev: 0,
});

export const room = $state({
  code: null as string | null,
  status: 'off' as 'off' | 'connecting' | 'online' | 'reconnecting',
  broker: '',
  members: [] as Member[],
  chat: [] as ChatMsg[],
  offset: 0,
  unread: 0,
});

export const pl = $state({
  pos: 0,
  dur: 0,
  loading: false,
  buffering: false,
  error: '',
});

export const recent: Track[] = $state(load<Track[]>('recent', []));

export const toasts = $state<{ id: string; text: string; kind: 'info' | 'error' }[]>([]);

export const bursts = $state<{ id: string; emoji: string; x: number; name: string }[]>([]);

export function toast(text: string, kind: 'info' | 'error' = 'info') {
  const id = rid(6);
  toasts.push({ id, text, kind });
  if (toasts.length > 4) toasts.shift();
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, kind === 'error' ? 5000 : 3200);
}

export function burst(emoji: string, name = '') {
  const id = rid(6);
  bursts.push({ id, emoji, name, x: 10 + Math.random() * 80 });
  setTimeout(() => {
    const i = bursts.findIndex((b) => b.id === id);
    if (i >= 0) bursts.splice(i, 1);
  }, 2600);
}

export function isHost() {
  return !room.code || ps.hostId === profile.uid;
}

/** Текущее время по часам комнаты (часам хоста). */
export function roomNow() {
  return Date.now() + (isHost() ? 0 : room.offset);
}

/** Где должен быть трек в момент `now` по часам хоста. */
export function posAt(s: PlayState, now: number) {
  if (!s.playing || !s.ready) return s.anchorPos;
  return s.anchorPos + (now - s.anchorTime) / 1000;
}

export function current(): QItem | null {
  return ps.queue[ps.index] ?? null;
}

export function pushRecent(t: Track) {
  const i = recent.findIndex((r) => r.id === t.id);
  if (i >= 0) recent.splice(i, 1);
  const { id, url, title, artist, duration, thumb, source, explicit } = t;
  recent.unshift({ id, url, title, artist, duration, thumb, source, explicit });
  if (recent.length > 30) recent.length = 30;
}

/** Сохраняем профиль, настройки и историю между запусками. */
export function persist() {
  $effect.root(() => {
    $effect(() => save('profile', $state.snapshot(profile)));
    $effect(() => save('settings', $state.snapshot(settings)));
    $effect(() => save('recent', $state.snapshot(recent)));
  });
}

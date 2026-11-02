import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Track } from './types';

export interface DepsStatus {
  ytdlp: boolean;
  fast: boolean;
  deno: boolean;
  version: string | null;
}

export interface DepsProgress {
  stage: 'yt-dlp' | 'deno';
  received: number;
  total: number;
}

export interface RemoteLyrics {
  synced: string | null;
  plain: string | null;
  instrumental: boolean;
  source: string;
}

export const api = {
  depsStatus: () => invoke<DepsStatus>('deps_status'),
  installDeps: () => invoke<void>('install_deps'),
  updateEngine: () => invoke<string>('update_engine'),
  search: (query: string, source: string) => invoke<Track[]>('search', { query, source }),
  /** hint — название и исполнитель: по ним ищется замена, если оригинал недоступен в регионе. */
  resolve: (url: string, force = false, hint?: { title: string; artist: string; duration: number }) =>
    invoke<{ key: string; duration: number; substitute: string | null }>('resolve', {
      url,
      force,
      hint: hint ? { title: hint.title, artist: hint.artist, duration: hint.duration } : null,
    }),
  resolveLink: (url: string) => invoke<Track[]>('resolve_link', { url }),
  onDepsProgress: (cb: (p: DepsProgress) => void) => listen<DepsProgress>('deps-progress', (e) => cb(e.payload)),
  streamUrl: (key: string) => convertFileSrc(key, 'stream'),
  lyrics: (title: string, artist: string, duration: number, url: string) =>
    invoke<RemoteLyrics | null>('lyrics', { title, artist, duration, url }),
  lyricsCandidates: (title: string, duration: number) =>
    invoke<import('./lyrics').Candidate[]>('lyrics_candidates', { title, duration }),
  /** Обложка через Rust (с кэшем в памяти). */
  img: (url: string) => (!url || !/^https?:/i.test(url) ? url : convertFileSrc(url, 'img')),
};

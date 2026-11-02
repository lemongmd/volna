/**
 * Предзагрузка: когда курсор задерживается на треке, заранее просим yt-dlp
 * достать ссылку на поток. К моменту клика она уже в кэше — трек стартует сразу.
 * Не больше двух одновременных запросов, чтобы не грузить процессор.
 */
import { api } from './api';
import type { Track } from './types';

const done = new Set<string>();
const queue: Track[] = [];
let active = 0;
let timer: ReturnType<typeof setTimeout> | undefined;
const MAX = 2;

function pump() {
  while (active < MAX && queue.length) {
    const t = queue.shift()!;
    const url = t.url;
    if (done.has(url)) continue;
    done.add(url);
    active++;
    api
      .resolve(url, false, t)
      .catch(() => done.delete(url))
      .finally(() => {
        active--;
        pump();
      });
  }
}

export function hoverStart(t: Track) {
  clearTimeout(timer);
  if (!t.url || done.has(t.url)) return;
  timer = setTimeout(() => {
    if (queue.some((q) => q.url === t.url)) return;
    queue.unshift(t);
    if (queue.length > 4) queue.length = 4;
    pump();
  }, 350);
}

export function hoverEnd() {
  clearTimeout(timer);
}

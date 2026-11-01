/**
 * Предзагрузка: когда курсор задерживается на треке, заранее просим yt-dlp
 * достать ссылку на поток. К моменту клика она уже в кэше — трек стартует сразу.
 * Не больше двух одновременных запросов, чтобы не грузить процессор.
 */
import { api } from './api';

const done = new Set<string>();
const queue: string[] = [];
let active = 0;
let timer: ReturnType<typeof setTimeout> | undefined;
const MAX = 2;

function pump() {
  while (active < MAX && queue.length) {
    const url = queue.shift()!;
    if (done.has(url)) continue;
    done.add(url);
    active++;
    api
      .resolve(url)
      .catch(() => done.delete(url))
      .finally(() => {
        active--;
        pump();
      });
  }
}

export function hoverStart(url: string) {
  clearTimeout(timer);
  if (!url || done.has(url)) return;
  timer = setTimeout(() => {
    if (queue.includes(url)) return;
    queue.unshift(url);
    if (queue.length > 4) queue.length = 4;
    pump();
  }, 350);
}

export function hoverEnd() {
  clearTimeout(timer);
}

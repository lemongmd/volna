/**
 * Палитра «Под обложку»: берём главный цвет обложки текущего трека и
 * окрашиваем им фон, ауру и акценты. Обложка грузится через наш протокол
 * img:// (он отдаёт CORS-заголовок), поэтому её можно прочитать в canvas.
 */
import { api } from './api';

const DEFAULT: [number, number, number] = [207, 227, 255];
const cache = new Map<string, [number, number, number]>();

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h = 0;
  if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
  else if (max === g) h = (b - r) / d + 2;
  else h = (r - g) / d + 4;
  return [h * 60, s, l];
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return [Math.round(f(0) * 255), Math.round(f(8) * 255), Math.round(f(4) * 255)];
}

/** Самый «живой» цвет картинки: группируем пиксели по оттенку, вес — насыщенность. */
function dominant(img: HTMLImageElement): [number, number, number] {
  const size = 32;
  const c = document.createElement('canvas');
  c.width = size;
  c.height = size;
  const ctx = c.getContext('2d', { willReadFrequently: true })!;
  ctx.drawImage(img, 0, 0, size, size);
  const data = ctx.getImageData(0, 0, size, size).data;
  const bins = Array.from({ length: 24 }, () => ({ w: 0, r: 0, g: 0, b: 0 }));
  let grey = { w: 0, r: 0, g: 0, b: 0 };
  for (let i = 0; i < data.length; i += 4) {
    const [r, g, b] = [data[i], data[i + 1], data[i + 2]];
    const [h, s, l] = rgbToHsl(r, g, b);
    if (l < 0.08 || l > 0.95 || s < 0.18) {
      grey = { w: grey.w + 1, r: grey.r + r, g: grey.g + g, b: grey.b + b };
      continue;
    }
    const w = s * (1 - Math.abs(l - 0.5));
    const bin = bins[Math.floor(h / 15) % 24];
    bin.w += w;
    bin.r += r * w;
    bin.g += g * w;
    bin.b += b * w;
  }
  const best = bins.reduce((a, b) => (b.w > a.w ? b : a));
  // почти бесцветная обложка — остаёмся в холодной «ауре»
  if (best.w < 6) return DEFAULT;
  return [best.r / best.w, best.g / best.w, best.b / best.w];
}

/** Нормализуем: на тёмном фоне цвет должен светиться, а не тонуть. */
function normalize([r, g, b]: [number, number, number]): [number, number, number] {
  const [h, s, l] = rgbToHsl(r, g, b);
  return hslToRgb(h, Math.max(0.55, Math.min(0.9, s)), Math.max(0.58, Math.min(0.72, l)));
}

function apply(rgb: [number, number, number]) {
  const [r, g, b] = rgb;
  const mixW = (v: number) => Math.round(v + (255 - v) * 0.55);
  const st = document.documentElement.style;
  st.setProperty('--cover', `rgb(${r}, ${g}, ${b})`);
  st.setProperty('--cover-glow', `rgb(${mixW(r)}, ${mixW(g)}, ${mixW(b)})`);
  st.setProperty('--cover-tint', `rgba(${r}, ${g}, ${b}, 0.34)`);
}

let lastSrc = '';

export function setCoverColor(thumb: string | undefined) {
  const src = thumb ?? '';
  if (src === lastSrc) return;
  lastSrc = src;
  if (!src) return apply(DEFAULT);
  const hit = cache.get(src);
  if (hit) return apply(hit);
  const img = new Image();
  img.crossOrigin = 'anonymous';
  img.decoding = 'async';
  img.onload = () => {
    if (lastSrc !== src) return;
    let rgb = DEFAULT;
    try {
      const d = dominant(img);
      rgb = d === DEFAULT ? DEFAULT : normalize(d);
    } catch {
      /* картинка без CORS — остаёмся на цвете по умолчанию */
    }
    cache.set(src, rgb);
    apply(rgb);
  };
  img.onerror = () => {
    if (lastSrc === src) apply(DEFAULT);
  };
  img.src = api.img(src);
}

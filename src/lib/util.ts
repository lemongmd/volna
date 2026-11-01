export function rid(n = 10): string {
  const a = 'abcdefghijklmnopqrstuvwxyz0123456789';
  const bytes = crypto.getRandomValues(new Uint8Array(n));
  let s = '';
  for (const b of bytes) s += a[b % a.length];
  return s;
}

export function roomCode(): string {
  const a = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  const bytes = crypto.getRandomValues(new Uint8Array(6));
  let s = '';
  for (const b of bytes) s += a[b % a.length];
  return s;
}

export function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem('volna:' + key);
    if (!raw) return fallback;
    const v = JSON.parse(raw);
    if (fallback && typeof fallback === 'object' && !Array.isArray(fallback)) return { ...fallback, ...v };
    return v as T;
  } catch {
    return fallback;
  }
}

export function save(key: string, value: unknown) {
  try {
    localStorage.setItem('volna:' + key, JSON.stringify(value));
  } catch {
    /* хранилище недоступно — не страшно */
  }
}

export function fmtTime(sec: number): string {
  if (!isFinite(sec) || sec < 0) sec = 0;
  const s = Math.floor(sec % 60);
  const m = Math.floor(sec / 60) % 60;
  const h = Math.floor(sec / 3600);
  const ss = s.toString().padStart(2, '0');
  return h ? `${h}:${m.toString().padStart(2, '0')}:${ss}` : `${m}:${ss}`;
}

export const COLORS = ['#a78bfa', '#f472b6', '#38bdf8', '#34d399', '#fbbf24', '#fb7185', '#c084fc', '#2dd4bf'];

export function pickColor() {
  return COLORS[Math.floor(Math.random() * COLORS.length)];
}

export function initials(name: string) {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (!parts.length) return '?';
  return (parts[0][0] + (parts[1]?.[0] ?? '')).toUpperCase();
}

export function isLink(s: string) {
  return /^(https?:\/\/|spotify:)/i.test(s.trim());
}

export function errText(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return 'Что-то пошло не так';
}

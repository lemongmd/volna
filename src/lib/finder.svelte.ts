/**
 * Переключатель поиска на Ctrl+F — по мотивам Alt+Tab.
 *
 * Держишь Ctrl и жмёшь F: появляется выбор «Треки / Строчки», каждое следующее нажатие F
 * переключает режим, отпустил Ctrl — открылся выбранный поиск. Быстрый Ctrl+F без удержания
 * сразу открывает режим по умолчанию, и переключатель даже не мелькает.
 *
 * Режим по умолчанию: «Строчки», если открыт текст песни на весь экран, иначе «Треки».
 */
import { current, ui } from './state.svelte';

export type FindMode = 'tracks' | 'lines';

export const finder = $state({
  /** Ctrl удержан и F уже нажата — идёт выбор. */
  active: false,
  /** Показан ли оверлей (появляется с небольшой задержкой). */
  open: false,
  sel: 'tracks' as FindMode,
});

/** Через сколько после первого Ctrl+F показывать оверлей, если Ctrl всё ещё зажат. */
const SHOW_DELAY = 220;
let timer: ReturnType<typeof setTimeout> | undefined;

/** Строчки ищем по тексту текущего трека — без трека искать там нечего. */
export function availableModes(): FindMode[] {
  return current() ? ['tracks', 'lines'] : ['tracks'];
}

export function defaultMode(): FindMode {
  return ui.view === 'now' && ui.npTab === 'lyrics' && current() ? 'lines' : 'tracks';
}

/** Нажатие Ctrl+F (back — с Shift, листаем в обратную сторону). */
export function findPress(back = false) {
  if (!finder.active) {
    finder.active = true;
    finder.sel = defaultMode();
    timer = setTimeout(() => {
      if (finder.active) finder.open = true;
    }, SHOW_DELAY);
    return;
  }
  findStep(back ? -1 : 1);
}

/** Листаем режимы и сразу показываем оверлей. */
export function findStep(dir: 1 | -1) {
  cycle(dir);
  finder.open = true;
  clearTimeout(timer);
}

export function cycle(dir: 1 | -1) {
  const modes = availableModes();
  const i = Math.max(0, modes.indexOf(finder.sel));
  finder.sel = modes[(i + dir + modes.length) % modes.length];
}

export function choose(mode: FindMode) {
  if (availableModes().includes(mode)) finder.sel = mode;
}

/** Ctrl отпущен (или клик по карточке): открываем выбранный поиск. */
export function findCommit() {
  if (!finder.active) return;
  const mode = finder.sel;
  reset();
  openFind(mode);
}

/** Esc: закрыть переключатель, ничего не открывая. */
export function findCancel() {
  reset();
}

function reset() {
  clearTimeout(timer);
  finder.active = false;
  finder.open = false;
}

export function openFind(mode: FindMode) {
  if (mode === 'lines' && current()) {
    ui.view = 'now';
    ui.npTab = 'lyrics';
    ui.lineFindReq = true;
  } else {
    ui.view = 'search';
    ui.trackFindReq = true;
  }
}

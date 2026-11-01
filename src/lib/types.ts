export type Source = 'ytm' | 'yt' | 'sc' | 'link';

export interface Track {
  id: string;
  url: string;
  title: string;
  artist: string;
  duration: number;
  thumb: string;
  source: Source;
  explicit?: boolean;
}

/** Элемент очереди: у одного трека может быть несколько копий в очереди. */
export interface QItem extends Track {
  qid: string;
  by?: string;
}

export type Repeat = 'off' | 'all' | 'one';

/** Общее состояние воспроизведения. В комнате им владеет хост. */
export interface PlayState {
  queue: QItem[];
  index: number;
  playing: boolean;
  /** Хост загрузил трек и реально начал играть. */
  ready: boolean;
  /** Позиция (сек) в момент anchorTime. */
  anchorPos: number;
  /** Время по часам хоста (мс). */
  anchorTime: number;
  hostId: string;
  hostOnly: boolean;
  repeat: Repeat;
  rev: number;
  last?: { id: string; uid: string; text: string };
}

export interface Member {
  uid: string;
  name: string;
  color: string;
  joinedAt: number;
}

export interface ChatMsg {
  id: string;
  uid: string;
  name: string;
  color: string;
  text: string;
  ts: number;
  system?: boolean;
}

export type Command =
  | { t: 'play' }
  | { t: 'pause' }
  | { t: 'toggle' }
  | { t: 'seek'; pos: number }
  | { t: 'jump'; i: number }
  | { t: 'next' }
  | { t: 'prev' }
  | { t: 'add'; items: QItem[]; mode: 'end' | 'next' | 'now' }
  | { t: 'remove'; qid: string }
  | { t: 'move'; from: number; to: number }
  | { t: 'clear' }
  | { t: 'shuffle' }
  | { t: 'ended'; qid: string }
  | { t: 'ready'; qid: string; pos: number }
  | { t: 'hostOnly'; v: boolean }
  | { t: 'repeat'; v: Repeat };

export interface By {
  uid: string;
  name: string;
}

export type View = 'home' | 'search' | 'now';

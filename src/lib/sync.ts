/**
 * Сердце «Волны»: общее состояние воспроизведения + комнаты.
 *
 * Как это устроено:
 *  - Каждая комната — набор MQTT-топиков `volna/v2/<КОД>/...` на публичном
 *    брокере (свой сервер не нужен; можно указать свой брокер в настройках).
 *  - Хост (создатель комнаты) — единственный источник правды. Он хранит очередь
 *    и «якорь» времени: «в момент T по моим часам трек был на позиции P».
 *    Состояние публикуется retained-сообщением, поэтому новый участник сразу
 *    получает актуальную картину.
 *  - Участники шлют хосту команды (play/pause/skip/add…), хост применяет их и
 *    рассылает новое состояние.
 *  - Часы участников сверяются с часами хоста (как NTP), поэтому все слышат
 *    один и тот же момент трека. Мелкий рассинхрон плеер выравнивает скоростью
 *    воспроизведения, крупный — перемоткой.
 *  - Если хост вышел, хостом автоматически становится следующий по времени входа.
 */
import type { MqttClient } from 'mqtt';
import type { By, ChatMsg, Command, Member, PlayState, QItem, Track } from './types';
import { burst, current, isHost, posAt, profile, ps, pushRecent, room, roomNow, settings, toast, ui } from './state.svelte';
import { rid, roomCode } from './util';
import * as player from './player';

export const BROKERS = [
  'wss://broker.emqx.io:8084/mqtt',
  'wss://broker.hivemq.com:8884/mqtt',
  'wss://test.mosquitto.org:8081/mqtt',
];

const MAX_QUEUE = 300;

let client: MqttClient | null = null;
let base = '';
let presenceReady = false;
let hostSeen = false;
let lastNotice = '';
let tsTimer: ReturnType<typeof setInterval> | undefined;
let tsSamples: { rtt: number; off: number }[] = [];
let hostCheckTimer: ReturnType<typeof setTimeout> | undefined;
let joinedAt = 0;
let stateWaiter: ((ok: boolean) => void) | null = null;

const me = () => profile.uid;
const myName = () => profile.name.trim() || 'Гость';

// ───────────────────────── применение команд ─────────────────────────

function hostPos(): number {
  const t = player.positionFor(current()?.qid);
  return t ?? Math.max(0, posAt(ps, roomNow()));
}

function startTrack(i: number, playing = true) {
  ps.index = i;
  ps.anchorPos = 0;
  ps.anchorTime = roomNow();
  ps.playing = playing;
  ps.ready = false;
}

function advance(): void {
  if (ps.index < ps.queue.length - 1) startTrack(ps.index + 1);
  else if (ps.repeat === 'all' && ps.queue.length) startTrack(0);
  else {
    ps.playing = false;
    ps.ready = false;
    ps.anchorPos = 0;
    ps.anchorTime = roomNow();
  }
}

function notice(by: By, text: string) {
  ps.last = { id: rid(6), uid: by.uid, text: `${by.name} ${text}` };
}

/** Применяет команду к состоянию. Выполняется только у хоста (или соло). */
function apply(c: Command, by: By): boolean {
  const now = roomNow();
  const cur = current();
  switch (c.t) {
    case 'play':
      if (!ps.queue.length) return false;
      if (ps.index < 0) {
        startTrack(0);
        break;
      }
      if (!ps.ready) {
        // трек ещё не стартовал (или очередь доиграла) — запускаем заново
        startTrack(ps.index);
        break;
      }
      ps.anchorPos = hostPos();
      ps.anchorTime = now;
      ps.playing = true;
      notice(by, 'включает музыку');
      break;
    case 'pause':
      if (!ps.playing) return false;
      ps.anchorPos = hostPos();
      ps.anchorTime = now;
      ps.playing = false;
      notice(by, 'ставит на паузу');
      break;
    case 'toggle':
      return apply({ t: ps.playing ? 'pause' : 'play' }, by);
    case 'seek': {
      if (!cur) return false;
      const max = cur.duration > 0 ? cur.duration - 0.5 : Infinity;
      ps.anchorPos = Math.max(0, Math.min(c.pos, max));
      ps.anchorTime = now;
      break;
    }
    case 'jump':
      if (c.i < 0 || c.i >= ps.queue.length) return false;
      startTrack(c.i);
      notice(by, `включает «${ps.queue[c.i].title}»`);
      break;
    case 'next':
      if (!ps.queue.length) return false;
      advance();
      notice(by, 'переключает трек');
      break;
    case 'prev':
      if (!cur) return false;
      if (hostPos() > 4 || ps.index === 0) {
        ps.anchorPos = 0;
        ps.anchorTime = now;
      } else startTrack(ps.index - 1);
      break;
    case 'add': {
      if (!c.items.length) return false;
      const room_ = MAX_QUEUE - ps.queue.length;
      const items = c.items.slice(0, Math.max(0, room_));
      if (!items.length) return false;
      if (ps.index < 0 || !ps.queue.length) {
        ps.queue.push(...items);
        startTrack(ps.queue.length - items.length);
      } else if (c.mode === 'end') {
        ps.queue.push(...items);
      } else {
        ps.queue.splice(ps.index + 1, 0, ...items);
        if (c.mode === 'now') startTrack(ps.index + 1);
      }
      notice(by, items.length === 1 ? `добавляет «${items[0].title}»` : `добавляет ${items.length} треков`);
      break;
    }
    case 'remove': {
      const i = ps.queue.findIndex((q) => q.qid === c.qid);
      if (i < 0) return false;
      ps.queue.splice(i, 1);
      if (i < ps.index) ps.index--;
      else if (i === ps.index) {
        if (!ps.queue.length) {
          ps.index = -1;
          ps.playing = false;
          ps.ready = false;
          ps.anchorPos = 0;
        } else if (ps.index >= ps.queue.length) {
          ps.index = ps.queue.length - 1;
          ps.playing = false;
          ps.ready = false;
          ps.anchorPos = 0;
        } else startTrack(ps.index, ps.playing);
      }
      break;
    }
    case 'move': {
      if (c.from === c.to || c.from < 0 || c.to < 0 || c.from >= ps.queue.length || c.to >= ps.queue.length)
        return false;
      const curQid = cur?.qid;
      const [it] = ps.queue.splice(c.from, 1);
      ps.queue.splice(c.to, 0, it);
      if (curQid) ps.index = ps.queue.findIndex((q) => q.qid === curQid);
      break;
    }
    case 'clear':
      if (cur) {
        ps.queue.splice(0, ps.queue.length, cur);
        ps.index = 0;
      } else {
        ps.queue.splice(0, ps.queue.length);
        ps.index = -1;
      }
      break;
    case 'shuffle': {
      const head = ps.queue.slice(0, ps.index + 1);
      const tail = ps.queue.slice(ps.index + 1);
      for (let i = tail.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [tail[i], tail[j]] = [tail[j], tail[i]];
      }
      ps.queue.splice(0, ps.queue.length, ...head, ...tail);
      break;
    }
    case 'ended':
      if (cur?.qid !== c.qid) return false;
      if (ps.repeat === 'one') {
        ps.anchorPos = 0;
        ps.anchorTime = now;
      } else advance();
      break;
    case 'ready':
      if (cur?.qid !== c.qid) return false;
      ps.ready = true;
      ps.anchorPos = c.pos;
      ps.anchorTime = now;
      break;
    case 'hostOnly':
      if (by.uid !== ps.hostId) return false;
      ps.hostOnly = c.v;
      break;
    case 'repeat':
      ps.repeat = c.v;
      break;
  }
  ps.rev++;
  return true;
}

/** Любое действие пользователя с плеером проходит через dispatch. */
export function dispatch(c: Command) {
  const by: By = { uid: me(), name: myName() };
  if (isHost()) {
    if (apply(c, by)) changed();
    return;
  }
  if (c.t === 'ready' || c.t === 'ended') return; // это внутренние события хоста
  if (ps.hostOnly && !(c.t === 'add' && c.mode === 'end')) {
    toast('Сейчас управляет только хост комнаты — можно добавлять треки в очередь');
    return;
  }
  publish('cmd', { from: me(), by, c });
}

export function makeItems(tracks: Track[]): QItem[] {
  return tracks.map((t) => ({ ...clone(t), qid: rid(8), by: myName() }));
}

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v));
}

export function playNow(t: Track) {
  dispatch({ t: 'add', items: makeItems([t]), mode: 'now' });
}
export function playNext(t: Track) {
  dispatch({ t: 'add', items: makeItems([t]), mode: 'next' });
  toast('Сыграет следующим');
}
export function addToQueue(tracks: Track[]) {
  dispatch({ t: 'add', items: makeItems(tracks), mode: 'end' });
  toast(tracks.length === 1 ? 'Добавлено в очередь' : `Добавлено треков: ${tracks.length}`);
}

function changed() {
  if (room.code && isHost()) publishState();
  afterStateChange();
}

let lastCurQid = '';
function afterStateChange() {
  const cur = current();
  if (cur && cur.qid !== lastCurQid) {
    lastCurQid = cur.qid;
    pushRecent(cur);
  }
  if (ps.last && ps.last.id !== lastNotice) {
    lastNotice = ps.last.id;
    if (room.code && ps.last.uid !== me()) toast(ps.last.text);
  }
  player.sync();
}

// ───────────────────────────── сеть ─────────────────────────────

function publish(sub: string, data: unknown, retain = false) {
  if (!client) return;
  const payload = data === null ? '' : JSON.stringify(data);
  client.publish(`${base}/${sub}`, payload, { qos: retain ? 1 : 0, retain });
}

function snapshot(): PlayState {
  return JSON.parse(JSON.stringify(ps));
}

function publishState() {
  publish('state', { from: me(), s: snapshot() }, true);
}

function publishPresence() {
  const m: Member = { uid: me(), name: myName(), color: profile.color, joinedAt };
  publish(`p/${me()}`, m, true);
  upsertMember(m);
}

/** Обновить имя/цвет в комнате после смены профиля. */
export function refreshPresence() {
  if (client && room.code) publishPresence();
}

// MQTT-клиент грузим лениво — только когда заходим в комнату.
async function tryConnect(url: string, code: string): Promise<MqttClient> {
  const { default: mqtt } = await import('mqtt');
  return new Promise((resolve, reject) => {
    const c = mqtt.connect(url, {
      clientId: `volna_${me()}_${rid(4)}`,
      clean: true,
      keepalive: 30,
      connectTimeout: 7000,
      reconnectPeriod: 0,
      protocolVersion: 4,
      will: { topic: `volna/v2/${code}/p/${me()}`, payload: '', qos: 1, retain: true },
    });
    const timer = setTimeout(() => {
      c.end(true);
      reject(new Error('timeout'));
    }, 8000);
    c.once('connect', () => {
      clearTimeout(timer);
      c.options.reconnectPeriod = 2500;
      resolve(c);
    });
    c.once('error', (e) => {
      clearTimeout(timer);
      c.end(true);
      reject(e);
    });
  });
}

async function connect(code: string) {
  base = `volna/v2/${code}`;
  room.code = code;
  room.status = 'connecting';
  room.members = [];
  room.chat = [];
  room.unread = 0;
  room.offset = 0;
  presenceReady = false;
  hostSeen = false;
  joinedAt = Date.now();

  const list = settings.broker.trim() ? [settings.broker.trim()] : BROKERS;
  let c: MqttClient | null = null;
  for (const url of list) {
    try {
      c = await tryConnect(url, code);
      room.broker = url;
      break;
    } catch {
      /* пробуем следующий брокер */
    }
  }
  if (!c) {
    room.code = null;
    room.status = 'off';
    throw new Error('Не удалось подключиться к серверу комнат. Проверь интернет.');
  }
  client = c;
  room.status = 'online';
  c.on('message', onMessage);
  c.on('connect', () => {
    room.status = 'online';
    publishPresence();
    if (isHost()) publishState();
  });
  c.on('reconnect', () => (room.status = 'reconnecting'));
  c.on('offline', () => (room.status = 'reconnecting'));
  await new Promise<void>((res, rej) =>
    c!.subscribe(`${base}/#`, { qos: 1 }, (err) => (err ? rej(err) : res())),
  );
  publishPresence();
  setTimeout(() => (presenceReady = true), 2000);
}

export async function createRoom() {
  if (room.code) await leaveRoom();
  const code = roomCode();
  await connect(code);
  ps.hostId = me();
  ps.hostOnly = false;
  ps.rev++;
  publishState();
  sysChat(`Комната ${code} создана. Отправь код друзьям!`);
  afterStateChange();
  return code;
}

export async function joinRoom(raw: string) {
  const code = raw.trim().toUpperCase().replace(/[^A-Z0-9]/g, '');
  if (code.length < 4) throw new Error('Неверный код комнаты');
  if (room.code === code) return;
  if (room.code) await leaveRoom();

  const prevHost = ps.hostId;
  ps.hostId = ''; // пока не знаем хоста — мы участник
  ps.playing = false;
  player.sync();
  try {
    await connect(code);
  } catch (e) {
    ps.hostId = prevHost;
    throw e;
  }
  const ok = await new Promise<boolean>((res) => {
    stateWaiter = res;
    setTimeout(() => res(false), 7000);
  });
  stateWaiter = null;
  if (!ok) {
    await disconnect();
    ps.hostId = me();
    afterStateChange();
    throw new Error('Комната не найдена');
  }
  sysChat(`Ты в комнате ${code}`);
  if (ps.hostId === me()) becomeHost(false);
  else {
    startTimeSync();
    // если хоста нет в сети (комната «умерла») — займём его место
    clearTimeout(hostCheckTimer);
    hostCheckTimer = setTimeout(checkHost, 3500);
  }
}

async function disconnect() {
  const c = client;
  client = null;
  clearInterval(tsTimer);
  clearTimeout(hostCheckTimer);
  if (c) await new Promise<void>((res) => c.end(false, {}, () => res()));
  room.code = null;
  room.status = 'off';
  room.members = [];
  room.offset = 0;
  room.unread = 0;
}

export async function leaveRoom() {
  if (!client) return;
  const wasHost = isHost();
  const others = room.members.filter((m) => m.uid !== me()).length;
  if (!wasHost) ps.anchorTime -= room.offset; // переводим якорь на свои часы
  publish(`p/${me()}`, null, true);
  if (wasHost && others === 0) publish('state', null, true);
  await disconnect();
  ps.hostId = me();
  ps.hostOnly = false;
  afterStateChange();
}

function onMessage(topic: string, payload: Uint8Array) {
  if (!topic.startsWith(base + '/')) return;
  const sub = topic.slice(base.length + 1);
  let data: any = null;
  const text = new TextDecoder().decode(payload);
  if (text) {
    try {
      data = JSON.parse(text);
    } catch {
      return;
    }
  }
  if (sub === 'state') onState(data);
  else if (sub === 'cmd') onCmd(data);
  else if (sub.startsWith('p/')) onPresence(sub.slice(2), data);
  else if (sub === 'chat') onChat(data);
  else if (sub === 't/req') onTimeReq(data);
  else if (sub === `t/res/${me()}`) onTimeRes(data);
  else if (sub === 'fx') onFx(data);
}

function onState(d: { from: string; s: PlayState } | null) {
  if (!d || !d.s || d.from === me()) return;
  if (isHost() && ps.hostId === me() && d.s.hostId !== me()) {
    // Два хоста (редкая гонка) — побеждает тот, кто зашёл раньше.
    const theirs = room.members.find((m) => m.uid === d.s.hostId)?.joinedAt ?? Infinity;
    if (theirs >= joinedAt) {
      publishState();
      return;
    }
  }
  const hostChanged = d.s.hostId !== ps.hostId;
  Object.assign(ps, d.s);
  if (!d.s.last) ps.last = undefined;
  if (hostChanged) {
    hostSeen = room.members.some((m) => m.uid === ps.hostId);
    if (ps.hostId !== me()) startTimeSync();
  }
  if (stateWaiter) {
    stateWaiter(true);
    stateWaiter = null;
  }
  afterStateChange();
}

function onCmd(d: { from: string; by: By; c: Command } | null) {
  if (!d || !isHost() || !d.c || !d.by) return;
  if (ps.hostOnly && d.by.uid !== me() && !(d.c.t === 'add' && d.c.mode === 'end')) return;
  if (d.c.t === 'ready' || d.c.t === 'ended' || d.c.t === 'hostOnly') return;
  if (apply(d.c, d.by)) changed();
}

function upsertMember(m: Member) {
  const i = room.members.findIndex((x) => x.uid === m.uid);
  if (i >= 0) room.members[i] = m;
  else room.members.push(m);
}

function onPresence(uid: string, d: Member | null) {
  const i = room.members.findIndex((m) => m.uid === uid);
  if (!d) {
    if (uid === me()) {
      // брокер мог стереть наше присутствие (обрыв связи) — восстановим
      if (client) publishPresence();
      return;
    }
    if (i >= 0) {
      const m = room.members[i];
      room.members.splice(i, 1);
      if (presenceReady) sysChat(`${m.name} выходит из комнаты`);
    }
    clearTimeout(hostCheckTimer);
    hostCheckTimer = setTimeout(checkHost, 600);
    return;
  }
  if (typeof d.uid !== 'string' || typeof d.name !== 'string') return;
  if (i < 0 && presenceReady && uid !== me()) sysChat(`${d.name} заходит в комнату`);
  upsertMember(d);
  if (uid === ps.hostId) hostSeen = true;
}

function checkHost() {
  if (!room.code || !client) return;
  if (room.members.some((m) => m.uid === ps.hostId)) return;
  const sorted = [...room.members].sort((a, b) => a.joinedAt - b.joinedAt || a.uid.localeCompare(b.uid));
  if (sorted[0]?.uid === me()) becomeHost(hostSeen);
}

function becomeHost(live: boolean) {
  if (live) {
    ps.anchorTime -= room.offset;
  } else {
    // Хозяин комнаты давно ушёл — начинаем текущий трек заново, на паузе.
    ps.playing = false;
    ps.ready = false;
    ps.anchorPos = 0;
    ps.anchorTime = Date.now();
  }
  ps.hostId = me();
  room.offset = 0;
  clearInterval(tsTimer);
  ps.rev++;
  publishState();
  sysChat('Теперь ты хост комнаты');
  afterStateChange();
}

// ───────────── синхронизация часов (как NTP, через брокер) ─────────────

function startTimeSync() {
  clearInterval(tsTimer);
  burstSync();
  tsTimer = setInterval(burstSync, 20000);
}

function burstSync() {
  tsSamples = [];
  for (let i = 0; i < 5; i++) {
    setTimeout(() => {
      if (!isHost()) publish('t/req', { from: me(), t0: Date.now() });
    }, i * 250);
  }
}

function onTimeReq(d: { from: string; t0: number } | null) {
  if (!d || !isHost() || d.from === me()) return;
  publish(`t/res/${d.from}`, { t0: d.t0, th: Date.now() });
}

function onTimeRes(d: { t0: number; th: number } | null) {
  if (!d || typeof d.t0 !== 'number' || typeof d.th !== 'number') return;
  const t1 = Date.now();
  const rtt = t1 - d.t0;
  if (rtt < 0 || rtt > 5000) return;
  tsSamples.push({ rtt, off: d.th + rtt / 2 - t1 });
  const best = tsSamples.reduce((a, b) => (b.rtt < a.rtt ? b : a));
  const prev = room.offset;
  room.offset = best.off;
  if (Math.abs(prev - best.off) > 150) player.align();
}

// ───────────────────────── чат и реакции ─────────────────────────

function sysChat(text: string) {
  room.chat.push({ id: rid(8), uid: '', name: '', color: '', text, ts: Date.now(), system: true });
  if (room.chat.length > 200) room.chat.splice(0, room.chat.length - 200);
}

export function sendChat(text: string) {
  const t = text.trim().slice(0, 500);
  if (!t || !client) return;
  const msg: ChatMsg = { id: rid(10), uid: me(), name: myName(), color: profile.color, text: t, ts: Date.now() };
  publish('chat', msg);
}

function onChat(d: ChatMsg | null) {
  if (!d || typeof d.text !== 'string' || room.chat.some((c) => c.id === d.id)) return;
  room.chat.push({ ...d, system: false });
  if (room.chat.length > 200) room.chat.splice(0, room.chat.length - 200);
  if (d.uid !== me() && !(ui.panel && ui.tab === 'chat')) room.unread++;
}

export function sendReaction(emoji: string) {
  if (client) publish('fx', { emoji, name: myName(), from: me() });
  else burst(emoji);
}

function onFx(d: { emoji: string; name: string } | null) {
  if (!d || typeof d.emoji !== 'string' || d.emoji.length > 8) return;
  burst(d.emoji, d.name);
}

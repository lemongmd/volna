<script lang="ts">
  import { tick } from 'svelte';
  import Icon from './Icon.svelte';
  import Cover from './Cover.svelte';
  import Avatar from './Avatar.svelte';
  import Bars from './Bars.svelte';
  import { isHost, profile, ps, room, ui } from '../lib/state.svelte';
  import { dispatch, leaveRoom, sendChat, sendReaction } from '../lib/sync';
  import { fmtTime } from '../lib/util';

  let text = $state('');
  let chatBox: HTMLDivElement | undefined = $state();
  let dragFrom = $state(-1);
  let dragOver = $state(-1);
  let copied = $state(false);

  const upcoming = $derived(ps.queue.length - Math.max(0, ps.index + 1));
  const hostName = $derived(room.members.find((m) => m.uid === ps.hostId)?.name ?? '');

  const TABS = [
    { id: 'queue', label: 'Очередь', icon: 'queue' },
    { id: 'chat', label: 'Чат', icon: 'chat' },
    { id: 'people', label: 'Люди', icon: 'users' },
  ] as const;

  $effect(() => {
    if (ui.tab === 'chat') room.unread = 0;
  });

  // автопрокрутка чата вниз
  $effect(() => {
    void room.chat.length;
    if (ui.tab === 'chat' && chatBox) tick().then(() => chatBox && (chatBox.scrollTop = chatBox.scrollHeight));
  });

  function submit(e: Event) {
    e.preventDefault();
    if (!text.trim()) return;
    sendChat(text);
    text = '';
  }

  function drop(i: number) {
    if (dragFrom >= 0 && dragFrom !== i) dispatch({ t: 'move', from: dragFrom, to: i });
    dragFrom = -1;
    dragOver = -1;
  }

  async function copyInvite() {
    if (!room.code) return;
    const msg = `Залетай ко мне в «Волну» 🌊 Код комнаты: ${room.code}`;
    await navigator.clipboard.writeText(msg).catch(() => {});
    copied = true;
    setTimeout(() => (copied = false), 1600);
  }

  function time(ts: number) {
    return new Date(ts).toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' });
  }
</script>

<aside class="panel glass">
  <div class="tabs">
    {#each TABS as t}
      <button class="tab" class:active={ui.tab === t.id} onclick={() => (ui.tab = t.id)}>
        <Icon name={t.icon} size={15} />
        {t.label}
        {#if t.id === 'chat' && room.unread}<span class="dot"></span>{/if}
      </button>
    {/each}
    <button class="icon-btn close" onclick={() => (ui.panel = false)} aria-label="Закрыть панель"><Icon name="x" size={16} /></button>
  </div>

  {#if ui.tab === 'queue'}
    <div class="q-head">
      <span class="muted">{ps.queue.length ? `Дальше: ${upcoming}` : 'Очередь пуста'}</span>
      <span class="grow"></span>
      <button class="icon-btn" onclick={() => dispatch({ t: 'shuffle' })} title="Перемешать" aria-label="Перемешать"><Icon name="shuffle" size={16} /></button>
      <button class="icon-btn" onclick={() => dispatch({ t: 'clear' })} title="Очистить" aria-label="Очистить"><Icon name="trash" size={16} /></button>
    </div>
    <div class="scroll" role="list">
      {#each ps.queue as q, i (q.qid)}
        <div
          class="q-row"
          class:cur={i === ps.index}
          class:past={i < ps.index}
          class:over={dragOver === i && dragFrom !== i}
          draggable="true"
          ondragstart={() => (dragFrom = i)}
          ondragover={(e) => {
            e.preventDefault();
            dragOver = i;
          }}
          ondragend={() => ((dragFrom = -1), (dragOver = -1))}
          ondrop={() => drop(i)}
          ondblclick={() => dispatch({ t: 'jump', i })}
          role="listitem"
        >
          <span class="grip"><Icon name="grip" size={14} /></span>
          <button class="q-cover" onclick={() => dispatch({ t: 'jump', i })} aria-label="Играть">
            <Cover src={q.thumb} size={42} radius={9} />
            {#if i === ps.index}<span class="q-eq"><Bars playing={ps.playing} size={14} /></span>{/if}
          </button>
          <div class="q-meta">
            <div class="ellipsis q-t">{q.title}</div>
            <div class="ellipsis q-a">{q.artist}{#if room.code && q.by} · {q.by}{/if}</div>
          </div>
          <span class="q-d">{q.duration ? fmtTime(q.duration) : ''}</span>
          <button class="icon-btn rm" onclick={() => dispatch({ t: 'remove', qid: q.qid })} aria-label="Убрать"><Icon name="x" size={14} /></button>
        </div>
      {:else}
        <div class="empty">
          <Icon name="queue" size={28} />
          <p>Добавляй треки кнопкой «+» —<br />{room.code ? 'очередь общая для всей комнаты' : 'они появятся здесь'}</p>
        </div>
      {/each}
    </div>
  {:else if !room.code}
    <div class="empty solo">
      <Icon name="users" size={30} />
      <p>Чат и список людей появятся, когда ты зайдёшь в комнату.</p>
      <button class="btn btn-accent" onclick={() => (ui.modal = 'join')}>Слушать вместе</button>
    </div>
  {:else if ui.tab === 'chat'}
    <div class="scroll chat" bind:this={chatBox}>
      {#each room.chat as m (m.id)}
        {#if m.system}
          <div class="sys">{m.text}</div>
        {:else}
          <div class="msg" class:mine={m.uid === profile.uid}>
            {#if m.uid !== profile.uid}<Avatar name={m.name} color={m.color} size={28} />{/if}
            <div class="bubble">
              {#if m.uid !== profile.uid}<div class="who" style="color:{m.color}">{m.name}</div>{/if}
              <div class="txt">{m.text}</div>
              <div class="ts">{time(m.ts)}</div>
            </div>
          </div>
        {/if}
      {/each}
    </div>
    <div class="quick">
      {#each ['🔥', '💜', '😂', '😭', '🤯', '👏'] as e}
        <button onclick={() => sendReaction(e)} aria-label="Реакция {e}">{e}</button>
      {/each}
    </div>
    <form class="compose" onsubmit={submit}>
      <input class="input" bind:value={text} placeholder="Сообщение…" maxlength="500" />
      <button class="send" type="submit" disabled={!text.trim()} aria-label="Отправить"><Icon name="send" size={17} /></button>
    </form>
  {:else}
    <div class="scroll">
      <div class="room-card">
        <div class="muted small">Код комнаты</div>
        <div class="rc-code">{room.code}</div>
        <button class="btn btn-primary full" onclick={copyInvite}>
          <Icon name={copied ? 'check' : 'copy'} size={16} />
          {copied ? 'Скопировано!' : 'Скопировать приглашение'}
        </button>
        <div class="conn muted small">
          <span class="cdot" class:warn={room.status !== 'online'}></span>
          {room.status === 'online' ? 'Подключено' : 'Переподключаемся…'}
        </div>
      </div>

      <div class="ppl-head muted small">В комнате · {room.members.length}</div>
      {#each room.members as m (m.uid)}
        <div class="person">
          <Avatar name={m.name} color={m.color} size={36} />
          <div class="p-meta">
            <div class="p-name">{m.name}{#if m.uid === profile.uid} <span class="muted">(ты)</span>{/if}</div>
            <div class="muted small">{m.uid === ps.hostId ? 'Хост комнаты' : 'Слушатель'}</div>
          </div>
          {#if m.uid === ps.hostId}<span class="crown"><Icon name="crown" size={16} /></span>{/if}
        </div>
      {/each}

      {#if isHost()}
        <label class="toggle">
          <span>
            <span class="t-title">Управляет только хост</span>
            <span class="muted small">Остальные могут добавлять треки в конец очереди, писать в чат и реагировать</span>
          </span>
          <input type="checkbox" checked={ps.hostOnly} onchange={(e) => dispatch({ t: 'hostOnly', v: (e.target as HTMLInputElement).checked })} />
          <span class="switch"></span>
        </label>
      {:else if ps.hostOnly}
        <p class="muted small note">Музыкой управляет {hostName || 'хост'}.</p>
      {/if}

      <button class="btn btn-glass full leave" onclick={() => leaveRoom()}>
        <Icon name="logout" size={16} /> Выйти из комнаты
      </button>
    </div>
  {/if}
</aside>

<style>
  .panel {
    width: 360px;
    flex: none;
    margin: 4px 16px 120px 0;
    border-radius: var(--r-lg);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slide-in 0.35s cubic-bezier(0.2, 0.8, 0.2, 1) both;
    background: linear-gradient(160deg, rgba(36, 24, 64, 0.6), rgba(16, 11, 30, 0.55));
  }
  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(24px);
    }
  }
  .tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 10px;
    border-bottom: 1px solid var(--stroke);
  }
  .tab {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: 99px;
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }
  .tab:hover {
    color: var(--text);
  }
  .tab.active {
    background: rgba(255, 255, 255, 0.14);
    color: var(--text);
    box-shadow: var(--hl);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #f472b6;
  }
  .close {
    margin-left: auto;
  }
  .q-head {
    display: flex;
    align-items: center;
    padding: 8px 10px 4px 16px;
    font-size: 13px;
  }
  .grow {
    flex: 1;
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 6px 8px 12px;
  }
  .q-row {
    display: grid;
    grid-template-columns: 14px 42px minmax(0, 1fr) auto 28px;
    align-items: center;
    gap: 10px;
    padding: 6px 6px 6px 4px;
    border-radius: 12px;
    border-top: 2px solid transparent;
  }
  .q-row:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .q-row.cur {
    background: linear-gradient(90deg, rgba(167, 139, 250, 0.25), rgba(167, 139, 250, 0.05));
  }
  .q-row.past {
    opacity: 0.5;
  }
  .q-row.over {
    border-top-color: var(--accent);
  }
  .grip {
    color: var(--faint);
    cursor: grab;
    opacity: 0;
  }
  .q-row:hover .grip {
    opacity: 1;
  }
  .q-cover {
    position: relative;
    border-radius: 9px;
  }
  .q-eq {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
    border-radius: 9px;
  }
  .q-meta {
    min-width: 0;
  }
  .q-t {
    font-weight: 600;
    font-size: 13px;
  }
  .cur .q-t {
    color: #e9d5ff;
  }
  .q-a {
    color: var(--muted);
    font-size: 12px;
  }
  .q-d {
    font-size: 12px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
  }
  .rm {
    width: 28px;
    height: 28px;
    opacity: 0;
  }
  .q-row:hover .rm {
    opacity: 1;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 6px;
    padding: 48px 20px;
    color: var(--muted);
  }
  .empty.solo {
    flex: 1;
    justify-content: center;
  }
  .empty p {
    margin: 4px 0 12px;
  }
  .chat {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    user-select: text;
  }
  .sys {
    align-self: center;
    font-size: 12px;
    color: var(--faint);
    padding: 4px 10px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.05);
    text-align: center;
  }
  .msg {
    display: flex;
    align-items: flex-end;
    gap: 8px;
    max-width: 88%;
  }
  .msg.mine {
    align-self: flex-end;
  }
  .bubble {
    padding: 8px 12px 6px;
    border-radius: 16px 16px 16px 4px;
    background: rgba(255, 255, 255, 0.09);
    border: 1px solid rgba(255, 255, 255, 0.08);
    min-width: 60px;
  }
  .mine .bubble {
    border-radius: 16px 16px 4px 16px;
    background: linear-gradient(135deg, rgba(139, 92, 246, 0.75), rgba(192, 132, 252, 0.6));
    border-color: rgba(255, 255, 255, 0.18);
  }
  .who {
    font-size: 12px;
    font-weight: 700;
    margin-bottom: 2px;
  }
  .txt {
    white-space: pre-wrap;
    word-break: break-word;
  }
  .ts {
    font-size: 10px;
    color: rgba(255, 255, 255, 0.5);
    text-align: right;
    margin-top: 2px;
  }
  .quick {
    display: flex;
    justify-content: space-around;
    padding: 6px 10px 0;
    border-top: 1px solid var(--stroke);
  }
  .quick button {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    font-size: 19px;
    transition: transform 0.15s, background 0.15s;
  }
  .quick button:hover {
    transform: scale(1.2);
    background: rgba(255, 255, 255, 0.1);
  }
  .compose {
    display: flex;
    gap: 8px;
    padding: 8px 10px 12px;
  }
  .compose .input {
    height: 42px;
    border-radius: 99px;
  }
  .send {
    width: 42px;
    height: 42px;
    flex: none;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, #8b5cf6, #c084fc);
    color: #fff;
  }
  .send:disabled {
    opacity: 0.4;
  }
  .room-card {
    margin: 6px 4px 18px;
    padding: 18px;
    border-radius: 20px;
    background: linear-gradient(140deg, rgba(124, 58, 237, 0.4), rgba(14, 165, 233, 0.15));
    border: 1px solid rgba(255, 255, 255, 0.14);
    text-align: center;
  }
  .rc-code {
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 34px;
    font-weight: 700;
    letter-spacing: 0.16em;
    margin: 4px 0 14px;
    user-select: text;
  }
  .full {
    width: 100%;
  }
  .conn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    margin-top: 10px;
  }
  .cdot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
  }
  .cdot.warn {
    background: #fbbf24;
  }
  .small {
    font-size: 12px;
  }
  .ppl-head {
    padding: 0 8px 6px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .person {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border-radius: 12px;
  }
  .person:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .p-meta {
    flex: 1;
    min-width: 0;
  }
  .p-name {
    font-weight: 600;
  }
  .crown {
    color: #fbbf24;
  }
  .toggle {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 16px 4px 0;
    padding: 14px;
    border-radius: 16px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--stroke);
    cursor: pointer;
  }
  .toggle > span:first-child {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .t-title {
    font-weight: 600;
  }
  .toggle input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  .switch {
    flex: none;
    width: 42px;
    height: 24px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.15);
    position: relative;
    transition: background 0.2s;
  }
  .switch::after {
    content: '';
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s;
  }
  .toggle input:checked + .switch {
    background: linear-gradient(135deg, #8b5cf6, #c084fc);
  }
  .toggle input:checked + .switch::after {
    transform: translateX(18px);
  }
  .note {
    margin: 14px 8px 0;
  }
  .leave {
    margin-top: 18px;
    color: #fda4af;
  }
</style>

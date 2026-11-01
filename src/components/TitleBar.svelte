<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';
  import { profile, room, ui } from '../lib/state.svelte';
  import type { View } from '../lib/types';

  const win = (() => {
    try {
      return getCurrentWindow();
    } catch {
      return null; // запуск в обычном браузере (разработка)
    }
  })();

  const nav: { id: View; label: string; icon: string }[] = [
    { id: 'home', label: 'Главная', icon: 'home' },
    { id: 'search', label: 'Поиск', icon: 'search' },
    { id: 'now', label: 'Сейчас играет', icon: 'disc' },
  ];

  function openRoom() {
    if (room.code) {
      ui.panel = true;
      ui.tab = 'people';
    } else ui.modal = 'join';
  }
</script>

<header class="bar" data-tauri-drag-region>
  <div class="left" data-tauri-drag-region>
    <button class="logo" onclick={() => (ui.view = 'home')} aria-label="Волна — на главную">
      <span class="mark">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="#fff" stroke-width="2.6" stroke-linecap="round"
          ><path d="M3 10c2-3.5 4-3.5 6 0s4 3.5 6 0 4-3.5 6 0" /><path d="M3 16c2-2.5 4-2.5 6 0s4 2.5 6 0 4-2.5 6 0" opacity=".55" /></svg
        >
      </span>
      <span class="word">Волна</span>
    </button>
  </div>

  <nav class="nav glass liquid" data-tauri-drag-region>
    {#each nav as n}
      <button class="tab" class:active={ui.view === n.id} onclick={() => (ui.view = n.id)}>
        <Icon name={n.icon} size={16} />
        {n.label}
      </button>
    {/each}
  </nav>

  <div class="right" data-tauri-drag-region>
    <button class="room-pill" class:live={!!room.code} onclick={openRoom}>
      {#if room.code}
        <span class="dot" class:warn={room.status !== 'online'}></span>
        <span class="code">{room.code}</span>
        <span class="stack">
          {#each room.members.slice(0, 4) as m (m.uid)}
            <Avatar name={m.name} color={m.color} size={22} ring />
          {/each}
        </span>
      {:else}
        <Icon name="users" size={16} />
        Слушать вместе
      {/if}
    </button>
    <button class="icon-btn" onclick={() => (ui.modal = 'settings')} aria-label="Настройки">
      <Icon name="settings" size={18} />
    </button>
    <button class="me" onclick={() => (ui.modal = 'settings')} aria-label="Профиль">
      <Avatar name={profile.name || 'Гость'} color={profile.color} size={30} />
    </button>
    {#if win}
      <div class="win">
        <button onclick={() => win.minimize()} aria-label="Свернуть"><Icon name="minimize" size={14} /></button>
        <button onclick={() => win.toggleMaximize()} aria-label="Развернуть"><Icon name="maximize" size={12} /></button>
        <button class="close" onclick={() => win.close()} aria-label="Закрыть"><Icon name="x" size={15} /></button>
      </div>
    {/if}
  </div>
</header>

<style>
  .bar {
    position: relative;
    z-index: 20;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0 0 0 20px;
    height: 60px;
  }
  .left,
  .right {
    display: flex;
    align-items: center;
    height: 100%;
    gap: 8px;
  }
  .right {
    justify-content: flex-end;
  }
  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .mark {
    width: 30px;
    height: 30px;
    border-radius: 10px;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, #7c3aed, #c084fc 60%, #38bdf8);
    box-shadow:
      0 6px 18px rgba(124, 58, 237, 0.5),
      inset 0 1px 0 rgba(255, 255, 255, 0.45);
  }
  .word {
    font-weight: 700;
    font-size: 17px;
    letter-spacing: -0.01em;
  }
  .nav {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: 999px;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 16px;
    border-radius: 999px;
    color: var(--muted);
    font-weight: 550;
    transition:
      background 0.25s,
      color 0.25s,
      box-shadow 0.25s;
  }
  .tab:hover {
    color: var(--text);
  }
  .tab.active {
    background: #fff;
    color: #140c28;
    box-shadow: 0 4px 18px rgba(255, 255, 255, 0.25);
  }
  .room-pill {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 14px;
    border-radius: 999px;
    font-weight: 600;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--stroke);
    box-shadow: var(--hl);
    transition: background 0.2s;
  }
  .room-pill:hover {
    background: rgba(255, 255, 255, 0.14);
  }
  .room-pill.live {
    background: linear-gradient(135deg, rgba(139, 92, 246, 0.35), rgba(192, 132, 252, 0.2));
    border-color: rgba(192, 132, 252, 0.45);
  }
  .code {
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    letter-spacing: 0.12em;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 10px var(--ok);
  }
  .dot.warn {
    background: #fbbf24;
    box-shadow: 0 0 10px #fbbf24;
  }
  .stack {
    display: flex;
  }
  .stack :global(.av + .av) {
    margin-left: -7px;
  }
  .me {
    margin: 0 6px 0 2px;
    border-radius: 50%;
  }
  .win {
    display: flex;
    height: 100%;
    margin-left: 4px;
  }
  .win button {
    width: 46px;
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--muted);
    transition: background 0.15s;
  }
  .win button:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
  }
  .win .close:hover {
    background: #e81123;
  }
</style>

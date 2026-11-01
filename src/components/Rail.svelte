<script lang="ts">
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';
  import { profile, room, ui } from '../lib/state.svelte';
  import type { View } from '../lib/types';

  const nav: { id: View; label: string; icon: string }[] = [
    { id: 'home', label: 'Главная', icon: 'home' },
    { id: 'search', label: 'Поиск', icon: 'search' },
    { id: 'now', label: 'Сейчас играет', icon: 'disc' },
  ];

  function openRoom() {
    if (room.code) {
      ui.panel = !(ui.panel && ui.tab === 'people');
      ui.tab = 'people';
    } else ui.modal = 'join';
  }
</script>

<nav class="rail" data-tauri-drag-region>
  <button class="logo" onclick={() => (ui.view = 'home')} aria-label="Волна" title="Волна">
    <svg viewBox="0 0 24 24" width="26" height="26" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round"
      ><path d="M2.5 10c2.2-3.8 4.3-3.8 6.5 0s4.3 3.8 6.5 0 4.3-3.8 6.5 0" /><path
        d="M2.5 16.5c2.2-2.8 4.3-2.8 6.5 0s4.3 2.8 6.5 0 4.3-2.8 6.5 0"
        opacity=".45"
      /></svg
    >
  </button>

  <div class="group surface-ish" data-tauri-drag-region>
    {#each nav as n}
      <button class="item" class:active={ui.view === n.id} onclick={() => (ui.view = n.id)} title={n.label} aria-label={n.label}>
        <Icon name={n.icon} size={20} />
      </button>
    {/each}
    <span class="sep"></span>
    <button class="item" class:active={!!room.code} onclick={openRoom} title={room.code ? `Комната ${room.code}` : 'Слушать вместе'} aria-label="Комната">
      <Icon name="users" size={20} />
      {#if room.code}<span class="live" class:warn={room.status !== 'online'}></span>{/if}
    </button>
  </div>

  <span class="grow" data-tauri-drag-region></span>

  <button class="item" onclick={() => (ui.modal = 'settings')} title="Настройки" aria-label="Настройки">
    <Icon name="settings" size={20} />
  </button>
  <button class="me" onclick={() => (ui.modal = 'settings')} title={profile.name || 'Профиль'} aria-label="Профиль">
    <Avatar name={profile.name || 'Гость'} color={profile.color} size={30} />
  </button>
</nav>

<style>
  .rail {
    position: relative;
    z-index: 2;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 12px 0 12px;
  }
  .logo {
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    color: var(--text);
    margin-bottom: 8px;
  }
  .group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 6px;
    border-radius: 14px;
    background: var(--panel);
    border: 1px solid var(--stroke);
  }
  .item {
    position: relative;
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    color: var(--muted);
    transition: background 0.15s, color 0.15s;
  }
  .item:hover {
    color: var(--text);
    background: var(--panel-2);
  }
  .item.active {
    color: var(--text);
    background: var(--panel-3);
  }
  .sep {
    height: 1px;
    margin: 2px 6px;
    background: var(--stroke-2);
  }
  .live {
    position: absolute;
    top: 7px;
    right: 7px;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--live);
    box-shadow: 0 0 0 2px #15151a;
  }
  .live.warn {
    background: #f5c04e;
  }
  .grow {
    flex: 1;
    align-self: stretch;
  }
  .me {
    margin-top: 4px;
    border-radius: 50%;
  }
</style>

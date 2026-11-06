<script lang="ts">
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';
  import { profile, room, ui } from '../lib/state.svelte';
  import type { View } from '../lib/types';

  const nav: { id: View; label: string; icon: string }[] = [
    { id: 'home', label: 'Главная', icon: 'home' },
    { id: 'search', label: 'Поиск', icon: 'search' },
    { id: 'now', label: 'Сейчас играет', icon: 'nowPlaying' },
  ];

  function openRoom() {
    if (room.code) {
      ui.panel = !(ui.panel && ui.tab === 'people');
      ui.tab = 'people';
    } else ui.modal = 'join';
  }
</script>

<nav class="side" data-tauri-drag-region>
  <button class="logo" onclick={() => (ui.view = 'home')} aria-label="Волна — на главную" title="Волна">
    <svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" aria-hidden="true"
      ><path d="M2.5 10c2.2-3.8 4.3-3.8 6.5 0s4.3 3.8 6.5 0 4.3-3.8 6.5 0" /><path
        d="M2.5 16.5c2.2-2.8 4.3-2.8 6.5 0s4.3 2.8 6.5 0 4.3-2.8 6.5 0"
        opacity=".45"
      /></svg
    >
    <span class="word display">волна</span>
  </button>

  <div class="nav">
    {#each nav as n}
      <button class="item" class:active={ui.view === n.id} onclick={() => (ui.view = n.id)} title={n.label} aria-label={n.label}>
        <Icon name={n.icon} size={20} />
        <span class="label">{n.label}</span>
      </button>
    {/each}
    <button class="item" class:active={!!room.code && ui.panel && ui.tab === 'people'} onclick={openRoom} title={room.code ? `Комната ${room.code}` : 'Слушать вместе'} aria-label="Комната">
      <span class="ico">
        <Icon name="users" size={20} />
        {#if room.code}<span class="live" class:warn={room.status !== 'online'}></span>{/if}
      </span>
      <span class="label">{room.code ? `Комната ${room.code}` : 'Слушать вместе'}</span>
    </button>
  </div>

  <span class="grow" data-tauri-drag-region></span>

  <div class="me">
    <button class="who" onclick={() => (ui.modal = 'settings')} title={profile.name || 'Профиль'} aria-label="Профиль">
      <Avatar name={profile.name || 'Гость'} color={profile.color} size={32} />
      <span class="label ellipsis">{profile.name || 'Гость'}</span>
    </button>
    <button class="icon-btn gear" onclick={() => (ui.modal = 'settings')} title="Настройки" aria-label="Настройки">
      <Icon name="settings" size={18} />
    </button>
  </div>
</nav>

<style>
  .side {
    position: relative;
    z-index: 2;
    display: flex;
    flex-direction: column;
    gap: var(--s2);
    padding: var(--s5) var(--s2) var(--s4);
    background: var(--side);
    border-right: 1px solid var(--stroke);
    min-width: 0;
  }
  .logo {
    display: flex;
    align-items: center;
    gap: var(--s2);
    padding: 0 var(--s3) var(--s5);
    color: var(--text);
  }
  .logo svg {
    flex: none;
  }
  .word {
    font-size: 26px;
    line-height: 1;
  }
  .nav {
    display: flex;
    flex-direction: column;
    gap: var(--s1);
  }
  .item {
    display: flex;
    align-items: center;
    gap: var(--s3);
    height: 48px;
    padding: 0 var(--s3);
    border-radius: var(--r);
    color: var(--muted);
    font-weight: 600;
    font-size: 14px;
    text-align: left;
    transition:
      background 0.15s,
      color 0.15s;
  }
  .item :global(svg) {
    flex: none;
  }
  .item:hover {
    color: var(--text);
    background: var(--panel);
  }
  .item.active {
    color: var(--text);
    background: var(--panel-2);
  }
  .ico {
    position: relative;
    display: grid;
  }
  .live {
    position: absolute;
    top: -2px;
    right: -3px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--live);
    box-shadow: 0 0 0 2px var(--side);
  }
  .live.warn {
    background: #f5c04e;
  }
  .label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .grow {
    flex: 1;
  }
  .me {
    display: flex;
    align-items: center;
    gap: var(--s1);
    padding-top: var(--s3);
    border-top: 1px solid var(--stroke);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: var(--s3);
    padding: var(--s1) var(--s2);
    border-radius: var(--r);
    font-weight: 600;
    text-align: left;
  }
  .who:hover {
    background: var(--panel);
  }
  .gear {
    flex: none;
  }

  /* узкое окно: только иконки */
  @media (max-width: 1100px) {
    .side {
      padding-inline: var(--s1);
      align-items: center;
    }
    .word,
    .label,
    .gear {
      display: none;
    }
    .logo {
      padding: 0 0 var(--s5);
    }
    .item {
      width: 48px;
      justify-content: center;
      padding: 0;
    }
    .me {
      flex-direction: column;
    }
    .who {
      padding: var(--s1);
    }
  }
</style>

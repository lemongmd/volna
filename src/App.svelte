<script lang="ts">
  import { onMount } from 'svelte';
  import Backdrop from './components/Backdrop.svelte';
  import Rail from './components/Rail.svelte';
  import WindowControls from './components/WindowControls.svelte';
  import PlayerBar from './components/PlayerBar.svelte';
  import SidePanel from './components/SidePanel.svelte';
  import Home from './components/Home.svelte';
  import Search from './components/Search.svelte';
  import NowPlaying from './components/NowPlaying.svelte';
  import Setup from './components/Setup.svelte';
  import RoomModal from './components/RoomModal.svelte';
  import SettingsModal from './components/SettingsModal.svelte';
  import Toasts from './components/Toasts.svelte';
  import { bursts, settings, ui } from './lib/state.svelte';
  import { setVolume } from './lib/player';
  import { dispatch } from './lib/sync';

  $effect(() => setVolume(settings.volume, settings.muted));

  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement;
      if (t.closest('input, textarea') || !ui.ready) return;
      if (e.code === 'Space') {
        e.preventDefault();
        dispatch({ t: 'toggle' });
      } else if (e.ctrlKey && e.code === 'ArrowRight') dispatch({ t: 'next' });
      else if (e.ctrlKey && e.code === 'ArrowLeft') dispatch({ t: 'prev' });
      else if (e.ctrlKey && e.code === 'KeyF') {
        e.preventDefault();
        ui.view = 'search';
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

<div class="app" class:fx={settings.fx}>
  <Backdrop />
  <Rail />

  <div class="column">
    <div class="drag" data-tauri-drag-region></div>
    <WindowControls />

    <div class="body">
      <main>
        {#key ui.view}
          <div class="view">
            {#if ui.view === 'home'}
              <Home />
            {:else if ui.view === 'search'}
              <Search />
            {:else}
              <NowPlaying />
            {/if}
          </div>
        {/key}
      </main>
      {#if ui.panel}
        <SidePanel />
      {/if}
    </div>

    <PlayerBar />
  </div>

  <div class="bursts" aria-hidden="true">
    {#each bursts as b (b.id)}
      <div class="burst" style="left:{b.x}%">
        <span class="emoji">{b.emoji}</span>
        {#if b.name}<span class="who">{b.name}</span>{/if}
      </div>
    {/each}
  </div>

  <Toasts />

  {#if ui.modal === 'join'}
    <RoomModal />
  {:else if ui.modal === 'settings'}
    <SettingsModal />
  {/if}

  {#if !ui.ready}
    <Setup />
  {/if}
</div>

<style>
  .app {
    position: relative;
    height: 100%;
    display: grid;
    grid-template-columns: 64px minmax(0, 1fr);
    overflow: hidden;
  }
  .column {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 0 8px 8px 0;
  }
  .drag {
    position: absolute;
    inset: 0 140px auto 0;
    height: 40px;
    z-index: 5;
  }
  .body {
    flex: 1;
    display: flex;
    gap: 8px;
    min-height: 0;
    margin-bottom: 8px;
  }
  main {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 44px 24px 28px 20px;
    container-type: inline-size;
  }
  .view {
    max-width: 1320px;
    margin: 0 auto;
    animation: fade-in 0.25s ease both;
  }
  .bursts {
    position: fixed;
    left: 64px;
    right: 0;
    bottom: 96px;
    height: 0;
    pointer-events: none;
    z-index: 60;
  }
  .burst {
    position: absolute;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    animation: float-up 2.6s cubic-bezier(0.2, 0.7, 0.3, 1) forwards;
  }
  .emoji {
    font-size: 36px;
  }
  .who {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.5);
    margin-top: 2px;
  }
  @keyframes float-up {
    0% {
      transform: translateY(0) scale(0.5);
      opacity: 0;
    }
    15% {
      transform: translateY(-30px) scale(1.1);
      opacity: 1;
    }
    100% {
      transform: translateY(-300px) scale(0.9);
      opacity: 0;
    }
  }
</style>

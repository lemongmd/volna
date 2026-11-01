<script lang="ts">
  import { onMount } from 'svelte';
  import Backdrop from './components/Backdrop.svelte';
  import TitleBar from './components/TitleBar.svelte';
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

<!-- SVG-фильтр для эффекта «жидкого стекла» (преломление краёв) -->
<svg width="0" height="0" style="position:absolute" aria-hidden="true">
  <filter id="liquid-glass" x="0%" y="0%" width="100%" height="100%" color-interpolation-filters="sRGB">
    <feTurbulence type="fractalNoise" baseFrequency="0.008 0.012" numOctaves="2" seed="7" result="noise" />
    <feGaussianBlur in="noise" stdDeviation="2" result="soft" />
    <feDisplacementMap in="SourceGraphic" in2="soft" scale="16" xChannelSelector="R" yChannelSelector="G" />
  </filter>
</svg>

<div class="app" class:fx={settings.fx}>
  <Backdrop />
  <TitleBar />

  <div class="body">
    <main class:with-panel={ui.panel}>
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
    grid-template-rows: 60px 1fr;
    overflow: hidden;
  }
  .body {
    position: relative;
    display: flex;
    min-height: 0;
    z-index: 1;
  }
  main {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 12px 36px 132px;
    scroll-behavior: smooth;
    container-type: inline-size;
  }
  .view {
    max-width: 1400px;
    margin: 0 auto;
    animation: fade-up 0.35s ease both;
  }
  .bursts {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 110px;
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
    font-size: 40px;
    filter: drop-shadow(0 6px 16px rgba(0, 0, 0, 0.4));
  }
  .who {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 99px;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(8px);
    margin-top: 2px;
  }
  @keyframes float-up {
    0% {
      transform: translateY(0) scale(0.4);
      opacity: 0;
    }
    15% {
      transform: translateY(-40px) scale(1.15);
      opacity: 1;
    }
    100% {
      transform: translateY(-340px) scale(0.9);
      opacity: 0;
    }
  }
</style>

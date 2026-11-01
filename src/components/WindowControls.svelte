<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Icon from './Icon.svelte';

  const win = (() => {
    try {
      return getCurrentWindow();
    } catch {
      return null; // запуск в обычном браузере (разработка)
    }
  })();
</script>

{#if win}
  <div class="wc glass">
    <button onclick={() => win.minimize()} aria-label="Свернуть"><Icon name="minimize" size={14} /></button>
    <button onclick={() => win.toggleMaximize()} aria-label="Развернуть"><Icon name="maximize" size={11} /></button>
    <button class="close" onclick={() => win.close()} aria-label="Закрыть"><Icon name="x" size={14} /></button>
  </div>
{/if}

<style>
  .wc {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 6;
    display: flex;
    padding: 3px;
    border-radius: 10px;
  }
  button {
    width: 34px;
    height: 26px;
    display: grid;
    place-items: center;
    border-radius: 7px;
    color: var(--muted);
    transition: background 0.15s, color 0.15s;
  }
  button:hover {
    background: var(--panel-2);
    color: var(--text);
  }
  .close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>

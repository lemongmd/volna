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
  <div class="wc">
    <button onclick={() => win.minimize()} aria-label="Свернуть"><Icon name="minimize" size={14} /></button>
    <button onclick={() => win.toggleMaximize()} aria-label="Развернуть"><Icon name="maximize" size={11} /></button>
    <button class="close" onclick={() => win.close()} aria-label="Закрыть"><Icon name="x" size={14} /></button>
  </div>
{/if}

<style>
  .wc {
    position: absolute;
    top: 0;
    right: 0;
    z-index: 6;
    display: flex;
  }
  button {
    width: 46px;
    height: 36px;
    display: grid;
    place-items: center;
    color: var(--muted);
    transition:
      background 0.15s,
      color 0.15s;
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

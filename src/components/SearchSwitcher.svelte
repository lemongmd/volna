<script lang="ts">
  import Icon from './Icon.svelte';
  import { current } from '../lib/state.svelte';
  import { availableModes, choose, finder, findCommit, type FindMode } from '../lib/finder.svelte';

  const cur = $derived(current());
  const enabled = $derived(availableModes());

  const MODES: { id: FindMode; icon: string; title: string; hint: string }[] = [
    { id: 'tracks', icon: 'music', title: 'Треки', hint: 'YouTube Music, YouTube и SoundCloud' },
    { id: 'lines', icon: 'lyrics', title: 'Строчки', hint: 'По тексту песни — и сразу перемотка' },
  ];
</script>

{#if finder.open}
  <div class="scrim" role="presentation" onmousedown={(e) => e.target === e.currentTarget && findCommit()}>
    <div class="switcher glass" role="listbox" aria-label="Режим поиска">
      <div class="cards">
        {#each MODES as m (m.id)}
          {@const off = !enabled.includes(m.id)}
          <button
            class="card"
            class:sel={finder.sel === m.id}
            class:off
            role="option"
            aria-selected={finder.sel === m.id}
            aria-disabled={off}
            onmouseenter={() => choose(m.id)}
            onclick={() => !off && (choose(m.id), findCommit())}
          >
            <span class="ic"><Icon name={m.icon} size={30} /></span>
            <span class="title">{m.title}</span>
            <span class="hint">
              {#if off}Ничего не играет{:else if m.id === 'lines' && cur}{cur.title}{:else}{m.hint}{/if}
            </span>
          </button>
        {/each}
      </div>
      <div class="keys">
        <span><kbd>Ctrl</kbd> + <kbd>F</kbd> — следующий режим</span>
        <span class="sep">·</span>
        <span>отпусти <kbd>Ctrl</kbd>, чтобы открыть</span>
        <span class="sep">·</span>
        <span><kbd>Esc</kbd> — отмена</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 150;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.42);
    animation: fade 0.14s ease both;
  }
  .switcher {
    padding: var(--s4);
    border-radius: var(--r-xl);
    background: rgba(26, 26, 33, 0.82);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.55);
    animation: pop 0.16s cubic-bezier(0.2, 0.8, 0.2, 1) both;
    max-width: calc(100vw - 32px);
  }
  .cards {
    display: flex;
    gap: var(--s3);
  }
  .card {
    position: relative;
    width: 220px;
    min-height: 148px;
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-end;
    gap: 4px;
    text-align: left;
    border-radius: var(--r-lg);
    background: var(--panel);
    border: 1px solid var(--stroke);
    color: var(--muted);
    transition:
      background 0.15s,
      color 0.15s,
      border-color 0.15s,
      transform 0.18s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .ic {
    margin-bottom: auto;
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: var(--pill);
    background: var(--panel-2);
    transition:
      background 0.15s,
      color 0.15s;
  }
  .title {
    font-family: var(--f-head);
    font-variation-settings: var(--head-axes);
    font-weight: 900;
    font-size: 24px;
    line-height: 1.1;
    color: var(--text);
  }
  .hint {
    max-width: 100%;
    font-size: 12.5px;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  /* выбранная карточка — «светлая», как активная кнопка в остальном интерфейсе */
  .card.sel {
    background: var(--light);
    color: rgba(11, 11, 13, 0.62);
    border-color: var(--light);
    transform: scale(1.04);
  }
  .card.sel .title {
    color: var(--light-ink);
  }
  .card.sel .ic {
    background: rgba(11, 11, 13, 0.1);
    color: var(--light-ink);
  }
  .card.off {
    opacity: 0.4;
    cursor: default;
  }
  .keys {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 6px;
    margin-top: var(--s4);
    font-size: 12px;
    color: var(--faint);
  }
  .sep {
    opacity: 0.6;
  }
  kbd {
    display: inline-block;
    min-width: 20px;
    padding: 1px 6px;
    border-radius: 5px;
    background: var(--panel-2);
    border: 1px solid var(--stroke-2);
    font: inherit;
    font-size: 11.5px;
    text-align: center;
    color: var(--muted);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.94) translateY(6px);
    }
  }
  @media (max-width: 560px) {
    .card {
      width: 150px;
      min-height: 124px;
    }
  }
</style>

<script lang="ts" generics="T extends string">
  import { tick } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    value,
    options,
    onchange,
    label,
    width = 160,
  }: {
    value: T;
    options: { value: T; label: string }[];
    onchange: (v: T) => void;
    label?: string;
    width?: number;
  } = $props();

  let open = $state(false);
  let hi = $state(0);
  let trigger: HTMLButtonElement | undefined = $state();
  let list: HTMLDivElement | undefined = $state();
  let pos = $state({ left: 0, top: 0, width: 0, up: false });

  const current = $derived(options.find((o) => o.value === value) ?? options[0]);
  const listId = `sel-${Math.random().toString(36).slice(2, 8)}`;

  /** Меню переезжает в <body>: у модалки есть backdrop-filter и overflow, они бы обрезали или сдвинули его. */
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }

  function place() {
    if (!trigger) return;
    const r = trigger.getBoundingClientRect();
    const h = Math.min(options.length * 38 + 12, 320);
    const up = window.innerHeight - r.bottom < h + 16 && r.top > window.innerHeight - r.bottom;
    pos = { left: r.left, top: up ? r.top - 6 : r.bottom + 6, width: r.width, up };
  }

  async function show() {
    hi = Math.max(0, options.findIndex((o) => o.value === value));
    place();
    open = true;
    await tick();
    list?.querySelector<HTMLElement>(`[data-i="${hi}"]`)?.scrollIntoView({ block: 'nearest' });
  }

  function hide(refocus = true) {
    open = false;
    if (refocus) trigger?.focus();
  }

  function pick(i: number) {
    const o = options[i];
    if (o && o.value !== value) onchange(o.value);
    hide();
  }

  function move(d: number) {
    hi = (hi + d + options.length) % options.length;
    list?.querySelector<HTMLElement>(`[data-i="${hi}"]`)?.scrollIntoView({ block: 'nearest' });
  }

  // Пока меню открыто, клавиши забираем в capture-фазе: Esc не должен закрывать ещё и модалку
  $effect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      const k = e.key;
      if (k === 'Escape') hide();
      else if (k === 'ArrowDown') move(1);
      else if (k === 'ArrowUp') move(-1);
      else if (k === 'Home') (hi = 0), move(0);
      else if (k === 'End') (hi = options.length - 1), move(0);
      else if (k === 'Enter' || k === ' ') pick(hi);
      else if (k === 'Tab') return hide(false);
      else return;
      e.preventDefault();
      e.stopPropagation();
    };
    const onDown = (e: MouseEvent) => {
      const n = e.target as Node;
      if (!list?.contains(n) && !trigger?.contains(n)) hide(false);
    };
    const onMove = () => place();
    const onScroll = (e: Event) => {
      if (!list?.contains(e.target as Node)) place();
    };
    window.addEventListener('keydown', onKey, true);
    window.addEventListener('mousedown', onDown, true);
    window.addEventListener('resize', onMove);
    window.addEventListener('scroll', onScroll, true);
    return () => {
      window.removeEventListener('keydown', onKey, true);
      window.removeEventListener('mousedown', onDown, true);
      window.removeEventListener('resize', onMove);
      window.removeEventListener('scroll', onScroll, true);
    };
  });
</script>

<button
  bind:this={trigger}
  type="button"
  class="input trigger"
  class:open
  style="width:{width}px"
  role="combobox"
  aria-haspopup="listbox"
  aria-expanded={open}
  aria-controls={listId}
  aria-label={label}
  onclick={(e) => {
    e.preventDefault();
    open ? hide(false) : show();
  }}
  onkeydown={(e) => {
    if (!open && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
      e.preventDefault();
      show();
    }
  }}
>
  <span class="ellipsis">{current?.label}</span>
  <span class="chev" class:flip={open}><Icon name="chevDown" size={14} /></span>
</button>

{#if open}
  <div
    use:portal
    bind:this={list}
    id={listId}
    class="menu"
    class:up={pos.up}
    role="listbox"
    aria-label={label}
    style="left:{pos.left}px; {pos.up ? `bottom:${window.innerHeight - pos.top}px` : `top:${pos.top}px`}; min-width:{pos.width}px"
  >
    {#each options as o, i (o.value)}
      <button
        type="button"
        class="opt"
        class:hi={i === hi}
        class:sel={o.value === value}
        role="option"
        aria-selected={o.value === value}
        data-i={i}
        onmouseenter={() => (hi = i)}
        onclick={() => pick(i)}
      >
        <span class="ellipsis">{o.label}</span>
        {#if o.value === value}<Icon name="check" size={15} />{/if}
      </button>
    {/each}
  </div>
{/if}

<style>
  .trigger {
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex: none;
    text-align: left;
    cursor: pointer;
    font-weight: 500;
  }
  .trigger:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .trigger.open {
    border-color: var(--stroke-2);
    background: rgba(255, 255, 255, 0.1);
  }
  .chev {
    display: grid;
    flex: none;
    color: var(--muted);
    transition: transform 0.18s ease;
  }
  .chev.flip {
    transform: rotate(180deg);
  }
  .menu {
    position: fixed;
    z-index: 300;
    max-height: 320px;
    overflow-y: auto;
    padding: 6px;
    border-radius: var(--r-lg);
    background: rgba(30, 30, 37, 0.97);
    border: 1px solid var(--stroke-2);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.55);
    color: var(--text);
    font-family: var(--f-ui);
    font-size: 14px;
    animation: drop 0.14s cubic-bezier(0.2, 0.8, 0.2, 1) both;
    transform-origin: top left;
  }
  .menu.up {
    transform-origin: bottom left;
  }
  .opt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    height: 38px;
    padding: 0 var(--s3);
    border-radius: var(--r-sm);
    text-align: left;
    color: var(--muted);
    transition: background 0.1s;
  }
  .opt.hi {
    background: var(--panel-2);
    color: var(--text);
  }
  .opt.sel {
    color: var(--text);
    font-weight: 600;
  }
  @keyframes drop {
    from {
      opacity: 0;
      transform: scale(0.97) translateY(-4px);
    }
  }
  .menu.up {
    animation-name: rise;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: scale(0.97) translateY(4px);
    }
  }
</style>

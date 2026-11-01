<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from './Modal.svelte';
  import Icon from './Icon.svelte';
  import { room, toast, ui } from '../lib/state.svelte';
  import { createRoom, joinRoom } from '../lib/sync';
  import { errText } from '../lib/util';

  let code = $state('');
  let busy = $state<'' | 'create' | 'join'>('');
  let error = $state('');
  let input: HTMLInputElement;

  onMount(() => input?.focus());

  function close() {
    if (!busy) ui.modal = null;
  }

  async function create() {
    busy = 'create';
    error = '';
    try {
      const c = await createRoom();
      await navigator.clipboard.writeText(c).catch(() => {});
      toast(`Комната ${c} создана — код скопирован`);
      ui.modal = null;
      ui.panel = true;
      ui.tab = 'people';
    } catch (e) {
      error = errText(e);
    } finally {
      busy = '';
    }
  }

  async function join(e?: Event) {
    e?.preventDefault();
    if (code.trim().length < 4) return;
    busy = 'join';
    error = '';
    try {
      await joinRoom(code);
      toast(`Ты в комнате ${room.code}`);
      ui.modal = null;
      ui.panel = true;
      ui.tab = 'chat';
    } catch (err) {
      error = errText(err);
    } finally {
      busy = '';
    }
  }

  function onCodeInput(e: Event) {
    const el = e.target as HTMLInputElement;
    code = el.value.toUpperCase().replace(/[^A-Z0-9]/g, '').slice(0, 6);
    el.value = code;
  }
</script>

<Modal onclose={close} width={480}>
  <div class="head">
    <h2>Слушать вместе</h2>
    <p class="muted">Все в комнате слышат один и тот же трек в одну и ту же секунду. Очередь, чат и реакции — общие.</p>
  </div>

  <button class="btn btn-primary big" onclick={create} disabled={!!busy}>
    {#if busy === 'create'}<span class="spinner"></span>{:else}<Icon name="plus" size={16} />{/if}
    Создать новую комнату
  </button>
  {#if room.code}
    <p class="muted small center">Ты сейчас в комнате {room.code} — она будет закрыта для тебя.</p>
  {/if}

  <div class="or"><span>или войди по коду</span></div>

  <form class="join" onsubmit={join}>
    <input
      bind:this={input}
      class="code-input"
      value={code}
      oninput={onCodeInput}
      placeholder="КОД"
      maxlength="6"
      spellcheck="false"
      autocomplete="off"
      aria-label="Код комнаты"
    />
    <button class="btn btn-lg" type="submit" disabled={code.length < 4 || !!busy}>
      {#if busy === 'join'}<span class="spinner"></span>{/if}
      Войти
    </button>
  </form>

  {#if error}<p class="error">{error}</p>{/if}
</Modal>

<style>
  .head {
    margin-bottom: 20px;
  }
  h2 {
    margin: 0 0 6px;
    font-size: 20px;
    font-weight: 650;
  }
  .head p {
    margin: 0;
    font-size: 13px;
    max-width: 380px;
  }
  .big {
    width: 100%;
    height: 42px;
  }
  .or {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 18px 0 14px;
    color: var(--faint);
    font-size: 12px;
  }
  .or::before,
  .or::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--stroke-2);
  }
  .join {
    display: flex;
    gap: 8px;
  }
  .code-input {
    flex: 1;
    min-width: 0;
    height: 42px;
    border-radius: var(--r-sm);
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid transparent;
    outline: none;
    text-align: center;
    font-family: 'Cascadia Mono', ui-monospace, Consolas, monospace;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: 0.3em;
    user-select: text;
  }
  .code-input:focus {
    border-color: var(--stroke-2);
  }
  .code-input::placeholder {
    color: var(--faint);
    letter-spacing: 0.2em;
  }
  .join .btn {
    height: 42px;
  }
  .error {
    margin: 12px 0 0;
    color: var(--danger);
    font-size: 13px;
  }
  .small {
    font-size: 12px;
  }
  .center {
    text-align: center;
    margin: 8px 0 0;
  }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';
  import { profile, ui } from '../lib/state.svelte';
  import { api, type DepsProgress } from '../lib/api';
  import { errText } from '../lib/util';

  // Первый запуск: имя + установка движка (yt-dlp и Deno, ~60 МБ один раз).
  let step = $state<'check' | 'name' | 'install' | 'error'>('check');
  let name = $state(profile.name);
  let progress = $state<DepsProgress | null>(null);
  let error = $state('');
  let needInstall = false;

  const pct = $derived(progress && progress.total ? Math.round((progress.received / progress.total) * 100) : 0);
  const mb = (n: number) => (n / 1024 / 1024).toFixed(1);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        unlisten = await api.onDepsProgress((p) => (progress = p));
        const st = await api.depsStatus();
        needInstall = !st.ytdlp || !st.deno || !st.fast;
      } catch {
        // не в Tauri (просто браузер) — пропускаем установку
        needInstall = false;
      }
      if (!profile.name) step = 'name';
      else await maybeInstall();
    })();
    return () => unlisten?.();
  });

  async function maybeInstall() {
    if (!needInstall) {
      ui.ready = true;
      return;
    }
    step = 'install';
    error = '';
    try {
      await api.installDeps();
      ui.ready = true;
    } catch (e) {
      error = errText(e);
      step = 'error';
    }
  }

  function saveName(e: Event) {
    e.preventDefault();
    const n = name.trim().slice(0, 24);
    if (!n) return;
    profile.name = n;
    maybeInstall();
  }
</script>

<div class="setup">
  <div class="card glass">
    <div class="logo">
      <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="#fff" stroke-width="2.4" stroke-linecap="round"
        ><path d="M3 10c2-3.5 4-3.5 6 0s4 3.5 6 0 4-3.5 6 0" /><path d="M3 16c2-2.5 4-2.5 6 0s4 2.5 6 0 4-2.5 6 0" opacity=".55" /></svg
      >
    </div>
    <h1>Волна</h1>

    {#if step === 'check'}
      <p class="muted">Секунду…</p>
      <span class="spinner"></span>
    {:else if step === 'name'}
      <p class="muted">Музыка вместе с друзьями. Как тебя будут видеть в комнатах?</p>
      <form onsubmit={saveName}>
        <!-- svelte-ignore a11y_autofocus -->
        <input class="input big" bind:value={name} maxlength="24" placeholder="Твоё имя" autofocus />
        <button class="btn btn-primary btn-lg wide" type="submit" disabled={!name.trim()}>
          Продолжить <Icon name="arrowRight" size={16} />
        </button>
      </form>
    {:else if step === 'install'}
      <p class="muted">Ставим движок воспроизведения. Это нужно один раз.</p>
      <div class="prog">
        <div class="prog-fill" style="width:{pct}%"></div>
      </div>
      <p class="small muted">
        {#if progress}
          {progress.stage === 'deno' ? 'JS-рантайм Deno' : 'yt-dlp'} · {mb(progress.received)}{progress.total
            ? ` из ${mb(progress.total)}`
            : ''} МБ
        {:else}
          Подключаемся…
        {/if}
      </p>
    {:else}
      <p class="err">Не получилось скачать движок: {error}</p>
      <p class="small muted">Проверь интернет. Если GitHub недоступен — включи VPN на минуту, это нужно только для первой установки.</p>
      <div class="row">
        <button class="btn btn-primary" onclick={maybeInstall}><Icon name="refresh" size={16} /> Повторить</button>
        <button class="btn" onclick={() => (ui.ready = true)}>Пропустить</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .setup {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: grid;
    place-items: center;
    background: var(--bg);
  }
  .card {
    width: min(380px, calc(100vw - 32px));
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    animation: fade-in 0.4s ease both;
  }
  h1 {
    margin: 14px 0 6px;
    font-size: 26px;
    font-weight: 700;
  }
  p {
    margin: 0 0 20px;
    font-size: 13.5px;
  }
  form {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .big {
    height: 44px;
    font-size: 15px;
    text-align: center;
  }
  .wide {
    width: 100%;
  }
  .prog {
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.12);
    overflow: hidden;
  }
  .prog-fill {
    height: 100%;
    background: #fff;
    transition: width 0.3s;
  }
  .small {
    font-size: 12px;
    margin-top: 12px;
  }
  .err {
    color: var(--danger);
  }
  .row {
    display: flex;
    gap: 8px;
  }
</style>

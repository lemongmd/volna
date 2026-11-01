<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';
  import { profile, settings, ui } from '../lib/state.svelte';
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
        if (settings.proxy) await api.setProxy(settings.proxy).catch(() => {});
        unlisten = await api.onDepsProgress((p) => (progress = p));
        const st = await api.depsStatus();
        needInstall = !st.ytdlp || !st.deno;
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
      <svg viewBox="0 0 24 24" width="34" height="34" fill="none" stroke="#fff" stroke-width="2.4" stroke-linecap="round"
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
        <button class="btn btn-primary wide" type="submit" disabled={!name.trim()}>
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
        <button class="btn btn-glass" onclick={() => (ui.ready = true)}>Пропустить</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .setup {
    position: fixed;
    inset: 60px 0 0;
    z-index: 200;
    display: grid;
    place-items: center;
    background: radial-gradient(800px 500px at 50% 30%, rgba(124, 58, 237, 0.35), transparent 70%), rgba(7, 6, 13, 0.92);
    backdrop-filter: blur(20px);
  }
  .card {
    width: min(440px, calc(100vw - 32px));
    padding: 40px 36px 34px;
    border-radius: 34px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    background: linear-gradient(150deg, rgba(48, 32, 84, 0.7), rgba(18, 12, 34, 0.7));
    animation: fade-up 0.5s ease both;
  }
  .logo {
    width: 76px;
    height: 76px;
    border-radius: 24px;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, #7c3aed, #c084fc 60%, #38bdf8);
    box-shadow: 0 20px 50px rgba(124, 58, 237, 0.55), inset 0 1px 0 rgba(255, 255, 255, 0.45);
  }
  h1 {
    margin: 18px 0 6px;
    font-size: 32px;
    letter-spacing: -0.02em;
  }
  p {
    margin: 0 0 20px;
  }
  form {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .big {
    height: 52px;
    font-size: 17px;
    text-align: center;
  }
  .wide {
    height: 48px;
  }
  .prog {
    width: 100%;
    height: 8px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }
  .prog-fill {
    height: 100%;
    border-radius: 99px;
    background: linear-gradient(90deg, #8b5cf6, #e9d5ff);
    box-shadow: 0 0 16px rgba(192, 132, 252, 0.7);
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
    gap: 10px;
  }
</style>

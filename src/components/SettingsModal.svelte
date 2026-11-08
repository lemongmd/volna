<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from './Modal.svelte';
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';
  import Select from './Select.svelte';
  import { profile, recent, room, settings, toast, ui } from '../lib/state.svelte';
  import { api, type DepsStatus } from '../lib/api';
  import { BROKERS, refreshPresence } from '../lib/sync';
  import { COLORS, errText } from '../lib/util';

  let name = $state(profile.name);
  let status = $state<DepsStatus | null>(null);
  let updating = $state(false);

  onMount(async () => {
    status = await api.depsStatus().catch(() => null);
  });

  function close() {
    const n = name.trim().slice(0, 24);
    if (n && n !== profile.name) {
      profile.name = n;
      refreshPresence();
    }
    ui.modal = null;
  }

  function setColor(c: string) {
    profile.color = c;
    refreshPresence();
  }

  const COOKIE_BROWSERS: { value: typeof settings.cookieBrowser; label: string }[] = [
    { value: 'none', label: 'Не использовать' },
    { value: 'chrome', label: 'Chrome' },
    { value: 'firefox', label: 'Firefox' },
    { value: 'edge', label: 'Edge' },
    { value: 'brave', label: 'Brave' },
  ];

  async function setCookieBrowser(browser: typeof settings.cookieBrowser) {
    settings.cookieBrowser = browser;
    try {
      await api.setCookieBrowser(browser);
    } catch (e) {
      toast(errText(e), 'error');
    }
  }

  async function update() {
    updating = true;
    try {
      const msg = await api.updateEngine();
      status = await api.depsStatus().catch(() => status);
      toast(msg);
    } catch (e) {
      toast(errText(e), 'error');
    } finally {
      updating = false;
    }
  }
</script>

<Modal onclose={close} width={520}>
  <h2>Настройки</h2>

  <section>
    <h3>Профиль</h3>
    <div class="profile">
      <Avatar name={name || 'Гость'} color={profile.color} size={48} />
      <div class="pf">
        <input class="input" bind:value={name} maxlength="24" placeholder="Как тебя зовут?" />
        <div class="colors">
          {#each COLORS as c}
            <button
              class="color"
              class:sel={profile.color === c}
              style="--c:{c}"
              onclick={() => setColor(c)}
              aria-label="Цвет {c}"
            ></button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section>
    <h3>Внешний вид</h3>
    <label class="row">
      <span>
        <span class="t">Фон под цвет обложки</span>
        <span class="muted s">Размытая обложка текущего трека на фоне. Выключи, если компьютер слабый.</span>
      </span>
      <input type="checkbox" bind:checked={settings.fx} />
      <span class="switch"></span>
    </label>
  </section>

  <section>
    <h3>Движок</h3>
    <div class="engine">
      <div>
        <div class="t">yt-dlp {status?.version ?? ''}</div>
        <div class="muted s">
          {#if status}
            {status.ytdlp ? (status.fast ? 'Установлен' : 'Установлен (медленная сборка — нажми «Обновить»)') : 'Не установлен'} · JS-рантайм {status.deno ? 'есть' : 'не найден'}
          {:else}
            Проверяем…
          {/if}
        </div>
      </div>
      <button class="btn" onclick={update} disabled={updating}>
        {#if updating}<span class="spinner"></span>{:else}<Icon name="refresh" size={16} />{/if}
        Обновить
      </button>
    </div>
    <p class="muted s">Если YouTube перестал играть — нажми «Обновить»: YouTube часто меняет защиту, и yt-dlp быстро выпускает фиксы.</p>
    <div class="row cookie-browser">
      <span>
        <span class="t">Cookies браузера</span>
        <span class="muted s">Использовать cookies выбранного браузера для yt-dlp.</span>
      </span>
      <Select value={settings.cookieBrowser} options={COOKIE_BROWSERS} onchange={setCookieBrowser} label="Cookies браузера" width={168} />
    </div>
  </section>

  <section>
    <h3>Сервер комнат</h3>
    <input class="input" bind:value={settings.broker} placeholder="Автоматически (публичные MQTT-брокеры)" spellcheck="false" />
    <p class="muted s">
      Адрес MQTT по WebSocket, например <code>wss://my-server:8084/mqtt</code>. У всех в комнате должен быть один и тот же
      сервер. Сейчас: {room.broker || BROKERS[0]}
    </p>
  </section>

  <section class="danger">
    <button class="btn" onclick={() => (recent.length = 0)}><Icon name="trash" size={16} /> Очистить историю</button>
  </section>
</Modal>

<style>
  h2 {
    margin: 0 0 14px;
    font-size: 20px;
    font-weight: 650;
  }
  section {
    padding: 14px 0;
    border-top: 1px solid var(--stroke-2);
  }
  h3 {
    margin: 0 0 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }
  .profile {
    display: flex;
    gap: 16px;
    align-items: center;
  }
  .pf {
    flex: 1;
  }
  .colors {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }
  .color {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--c);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.4);
    transition: transform 0.15s;
  }
  .color:hover {
    transform: scale(1.15);
  }
  .color.sel {
    outline: 2px solid #fff;
    outline-offset: 2px;
  }
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
  }
  .row > span:first-child {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .t {
    font-weight: 600;
  }
  .s {
    font-size: 12px;
  }
  .row input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  .switch {
    flex: none;
    width: 36px;
    height: 20px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.15);
    position: relative;
    transition: background 0.2s;
  }
  .switch::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s;
  }
  .row input:checked + .switch {
    background: var(--live);
  }
  .row input:checked + .switch::after {
    transform: translateX(16px);
  }
  .engine {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .cookie-browser {
    margin-top: 12px;
    cursor: default;
  }
  p.s {
    margin: 10px 0 0;
  }
  code {
    font-family: ui-monospace, Consolas, monospace;
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 5px;
    border-radius: 5px;
  }
  .danger {
    padding-bottom: 0;
  }
</style>

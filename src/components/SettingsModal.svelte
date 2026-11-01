<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from './Modal.svelte';
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';
  import { profile, recent, room, settings, toast, ui } from '../lib/state.svelte';
  import { api, type DepsStatus } from '../lib/api';
  import { BROKERS, refreshPresence } from '../lib/sync';
  import { COLORS, errText } from '../lib/util';

  let name = $state(profile.name);
  let proxy = $state(settings.proxy);
  let proxyState = $state<'' | 'ok' | 'err'>('');

  async function applyProxy() {
    const v = proxy.trim();
    if (v === settings.proxy) return;
    try {
      await api.setProxy(v);
      settings.proxy = v;
      proxyState = 'ok';
      toast(v ? 'Прокси включён' : 'Прокси выключен');
    } catch (e) {
      proxyState = 'err';
      toast(errText(e), 'error');
    }
  }
  let status = $state<DepsStatus | null>(null);
  let updating = $state(false);

  onMount(async () => {
    status = await api.depsStatus().catch(() => null);
  });

  function close() {
    void applyProxy();
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
      <Avatar name={name || 'Гость'} color={profile.color} size={56} />
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
        <span class="t">Живые эффекты</span>
        <span class="muted s">Анимация фона и «жидкое стекло». Выключи, если компьютер слабый.</span>
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
            {status.ytdlp ? 'Установлен' : 'Не установлен'} · JS-рантайм {status.deno ? 'есть' : 'не найден'}
          {:else}
            Проверяем…
          {/if}
        </div>
      </div>
      <button class="btn btn-glass" onclick={update} disabled={updating}>
        {#if updating}<span class="spinner"></span>{:else}<Icon name="refresh" size={16} />{/if}
        Обновить
      </button>
    </div>
    <p class="muted s">Если YouTube перестал играть — нажми «Обновить»: YouTube часто меняет защиту, и yt-dlp быстро выпускает фиксы.</p>
  </section>

  <section>
    <h3>Прокси</h3>
    <input
      class="input"
      class:bad={proxyState === 'err'}
      bind:value={proxy}
      onchange={applyProxy}
      placeholder="Без прокси"
      spellcheck="false"
    />
    <p class="muted s">
      Если YouTube тормозит или не открывается — укажи прокси, например <code>socks5://127.0.0.1:1080</code> или
      <code>http://127.0.0.1:2080</code>. Через него пойдут поиск, загрузка и сам звук. Комнаты работают и без него.
    </p>
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
    <button class="btn btn-glass" onclick={() => (recent.length = 0)}><Icon name="trash" size={16} /> Очистить историю</button>
  </section>
</Modal>

<style>
  h2 {
    margin: 0 0 18px;
    font-size: 24px;
  }
  section {
    padding: 16px 0;
    border-top: 1px solid var(--stroke);
  }
  h3 {
    margin: 0 0 12px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--faint);
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
    width: 42px;
    height: 24px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.15);
    position: relative;
    transition: background 0.2s;
  }
  .switch::after {
    content: '';
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s;
  }
  .row input:checked + .switch {
    background: linear-gradient(135deg, #8b5cf6, #c084fc);
  }
  .row input:checked + .switch::after {
    transform: translateX(18px);
  }
  .engine {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
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
  .input.bad {
    border-color: var(--danger);
  }
  .danger {
    padding-bottom: 0;
  }
</style>

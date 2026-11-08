import { mount } from 'svelte';
import '@fontsource-variable/roboto-flex/full.css';
import '@fontsource-variable/onest/wght.css';
import './app.css';
import App from './App.svelte';
import { persist, settings } from './lib/state.svelte';
import { api } from './lib/api';
import * as player from './lib/player';

persist();

async function initialize() {
  try {
    await api.setCookieBrowser(settings.cookieBrowser ?? 'none');
  } catch (e) {
    console.error('Не удалось применить настройки cookies браузера:', e);
  }
  player.init();
}

void initialize();

// Отключаем системное меню WebView по правому клику (кроме полей ввода).
window.addEventListener('contextmenu', (e) => {
  const t = e.target as HTMLElement;
  if (!t.closest('input, textarea')) e.preventDefault();
});

export default mount(App, { target: document.getElementById('app')! });

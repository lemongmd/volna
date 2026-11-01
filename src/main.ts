import { mount } from 'svelte';
import '@fontsource-variable/geist/wght.css';
import './app.css';
import App from './App.svelte';
import { persist } from './lib/state.svelte';
import * as player from './lib/player';

persist();
player.init();

// Отключаем системное меню WebView по правому клику (кроме полей ввода).
window.addEventListener('contextmenu', (e) => {
  const t = e.target as HTMLElement;
  if (!t.closest('input, textarea')) e.preventDefault();
});

export default mount(App, { target: document.getElementById('app')! });

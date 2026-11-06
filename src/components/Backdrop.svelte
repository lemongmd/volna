<script lang="ts">
  import { current, settings } from '../lib/state.svelte';
  import { setCoverColor } from '../lib/coverColor';

  // Палитра «Под обложку»: главный цвет обложки заливает фон мягким светом.
  $effect(() => setCoverColor(current()?.thumb));
</script>

<div class="backdrop" class:off={!settings.fx} aria-hidden="true">
  <div class="flood"></div>
  <div class="shade"></div>
</div>

<style>
  .backdrop {
    position: absolute;
    inset: 0;
    z-index: 0;
    overflow: hidden;
    pointer-events: none;
    background: var(--bg);
  }
  .flood {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(70% 60% at 62% 38%, var(--cover-tint), transparent 70%),
      radial-gradient(45% 40% at 18% 100%, var(--cover-tint), transparent 75%);
    opacity: 0.9;
    transition: opacity 0.6s ease;
  }
  .off .flood {
    opacity: 0.25;
  }
  .shade {
    position: absolute;
    inset: 0;
    background: linear-gradient(180deg, rgba(7, 7, 8, 0) 0%, rgba(7, 7, 8, 0.55) 100%);
  }
</style>

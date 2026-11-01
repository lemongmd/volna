<script lang="ts">
  import { initials } from '../lib/util';
  let { name, color, size = 32, ring = false }: { name: string; color: string; size?: number; ring?: boolean } =
    $props();
</script>

<span
  class="av"
  class:ring
  style="width:{size}px;height:{size}px;font-size:{size * 0.38}px;--c:{color}"
  title={name}
>
  {initials(name)}
</span>

<style>
  .av {
    display: inline-grid;
    place-items: center;
    flex: none;
    border-radius: 50%;
    font-weight: 700;
    color: #fff;
    background: radial-gradient(circle at 30% 25%, color-mix(in srgb, var(--c) 60%, #fff), var(--c) 55%, color-mix(in srgb, var(--c) 60%, #000));
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.4),
      0 4px 12px color-mix(in srgb, var(--c) 40%, transparent);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }
  .ring {
    outline: 2px solid #0c0817;
  }
</style>

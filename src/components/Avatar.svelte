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
    background: color-mix(in srgb, var(--c) 78%, #000);
    text-shadow: 0 1px 1px rgba(0, 0, 0, 0.25);
  }
  .ring {
    outline: 2px solid #17171c;
  }
</style>

<script lang="ts">
  let {
    value,
    max = 1,
    step = 0.01,
    disabled = false,
    label = '',
    oninput,
    onchange,
  }: {
    value: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    label?: string;
    oninput?: (v: number) => void;
    onchange?: (v: number) => void;
  } = $props();

  const pct = $derived(max ? Math.max(0, Math.min(100, (value / max) * 100)) : 0);
</script>

<div class="slider" class:disabled style="--p:{pct}%">
  <div class="track"><div class="fill"></div></div>
  <input
    type="range"
    min="0"
    {max}
    {step}
    {value}
    {disabled}
    aria-label={label}
    oninput={(e) => oninput?.(+(e.target as HTMLInputElement).value)}
    onchange={(e) => onchange?.(+(e.target as HTMLInputElement).value)}
  />
</div>

<style>
  .slider {
    position: relative;
    flex: 1;
    min-width: 0;
    height: 16px;
    display: flex;
    align-items: center;
  }
  .track {
    position: absolute;
    left: 0;
    right: 0;
    height: 3px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.16);
    overflow: hidden;
    transition: height 0.12s;
  }
  .fill {
    width: var(--p);
    height: 100%;
    /* прогресс — в цвете обложки */
    background: var(--cover-glow);
  }
  .slider:hover .track {
    height: 5px;
  }
  .slider:hover .fill {
    background: var(--cover);
  }
  input {
    position: relative;
    width: 100%;
    height: 16px;
    margin: 0;
    background: transparent;
    appearance: none;
    cursor: pointer;
  }
  input::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #fff;
    opacity: 0;
    transition: opacity 0.12s;
  }
  .slider:hover input::-webkit-slider-thumb {
    opacity: 1;
  }
  .disabled input {
    cursor: default;
  }
  .disabled .fill {
    opacity: 0;
  }
</style>

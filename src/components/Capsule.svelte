<script lang="ts">
  import { fmtTime } from '../lib/util';

  // Капсула: название трека в центре, прогресс заливает фон; клик и перетаскивание — перемотка.
  let {
    pos,
    dur,
    label,
    onseek,
    disabled = false,
  }: { pos: number; dur: number; label: string; onseek: (v: number) => void; disabled?: boolean } = $props();

  let el: HTMLDivElement;
  let drag = $state<number | null>(null);
  const shown = $derived(drag ?? Math.min(pos, dur || pos));
  const pct = $derived(dur ? Math.max(0, Math.min(100, (shown / dur) * 100)) : 0);

  function at(e: PointerEvent) {
    const r = el.getBoundingClientRect();
    return Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * dur;
  }
  function down(e: PointerEvent) {
    if (disabled || !dur) return;
    el.setPointerCapture(e.pointerId);
    drag = at(e);
  }
  function move(e: PointerEvent) {
    if (drag !== null) drag = at(e);
  }
  function up() {
    if (drag === null) return;
    onseek(drag);
    drag = null;
  }
  function key(e: KeyboardEvent) {
    if (disabled || !dur) return;
    if (e.key === 'ArrowRight') onseek(Math.min(dur, shown + 5));
    else if (e.key === 'ArrowLeft') onseek(Math.max(0, shown - 5));
    else return;
    e.preventDefault();
  }
</script>

<div
  class="capsule"
  class:disabled
  bind:this={el}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={() => (drag = null)}
  onkeydown={key}
  role="slider"
  tabindex="0"
  aria-label="Позиция трека"
  aria-valuemin="0"
  aria-valuemax={Math.round(dur)}
  aria-valuenow={Math.round(shown)}
  aria-valuetext={`${fmtTime(shown)} из ${fmtTime(dur)}`}
>
  <div class="fill" style="width:{pct}%"></div>
  <span class="t">{fmtTime(shown)}</span>
  <span class="label">{label}</span>
  <span class="t">{fmtTime(dur)}</span>
</div>

<style>
  .capsule {
    position: relative;
    flex: 1;
    min-width: 0;
    height: 56px;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s3);
    padding: 0 var(--s5);
    border-radius: var(--pill);
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: rgba(12, 12, 14, 0.72);
    overflow: hidden;
    cursor: pointer;
    touch-action: none;
  }
  .capsule:hover {
    border-color: rgba(255, 255, 255, 0.35);
  }
  .capsule:focus-visible {
    outline: 2px solid var(--cover-glow);
    outline-offset: 2px;
  }
  .disabled {
    cursor: default;
  }
  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: color-mix(in srgb, var(--cover) 22%, transparent);
    border-right: 2px solid var(--cover-glow);
    pointer-events: none;
    transition: width 0.25s linear;
  }
  .label {
    position: relative;
    text-align: center;
    font-weight: 700;
    font-size: 15px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .t {
    position: relative;
    font-size: 12px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
</style>

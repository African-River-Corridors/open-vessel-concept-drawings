<script lang="ts">
  // One hull view. The SVG is made in Rust (the drawing crate, to the Drawing Format Standard); this
  // component only places it and lets the user zoom and move it. It holds no geometry and no scale.
  let { svg, label }: { svg: string; label: string } = $props();

  const MIN = 1;
  const MAX = 12;
  let zoom = $state(1);
  let x = $state(0);
  let y = $state(0);
  let frame: HTMLDivElement;

  /** Keep the sheet in view: it can be moved only as far as its edge reaches the frame edge. */
  function clamp() {
    const w = frame.clientWidth;
    const h = frame.clientHeight;
    x = Math.min(0, Math.max(w - w * zoom, x));
    y = Math.min(0, Math.max(h - h * zoom, y));
  }

  /** Zoom about a point of the frame, so that the point stays under the pointer. */
  function zoomAt(factor: number, px: number, py: number) {
    const next = Math.min(MAX, Math.max(MIN, zoom * factor));
    const k = next / zoom;
    x = px - (px - x) * k;
    y = py - (py - y) * k;
    zoom = next;
    clamp();
  }

  /** Ctrl (or Cmd) and the wheel, or a pinch, zooms. A plain wheel scrolls the panel as usual. */
  function onWheel(e: WheelEvent) {
    if (!(e.ctrlKey || e.metaKey)) return;
    e.preventDefault();
    const box = frame.getBoundingClientRect();
    zoomAt(Math.exp(-e.deltaY * 0.002), e.clientX - box.left, e.clientY - box.top);
  }

  // A listener that may cancel the page's own scrolling cannot be passive.
  $effect(() => {
    frame.addEventListener("wheel", onWheel, { passive: false });
    return () => frame.removeEventListener("wheel", onWheel);
  });

  function onDown(e: PointerEvent) {
    if (zoom === 1) return;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
    let lastX = e.clientX;
    let lastY = e.clientY;
    const move = (m: PointerEvent) => {
      x += m.clientX - lastX;
      y += m.clientY - lastY;
      lastX = m.clientX;
      lastY = m.clientY;
      clamp();
    };
    const up = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", up);
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", up);
  }

  function step(factor: number) {
    zoomAt(factor, frame.clientWidth / 2, frame.clientHeight / 2);
  }

  function fit() {
    zoom = 1;
    x = 0;
    y = 0;
  }
</script>

<div class="wrap">
  <div class="tools" role="group" aria-label="Zoom {label}">
    <button type="button" aria-label="Zoom out" onclick={() => step(1 / 1.5)} disabled={zoom <= MIN}>−</button>
    <button type="button" aria-label="Zoom in" onclick={() => step(1.5)} disabled={zoom >= MAX}>+</button>
    <button type="button" aria-label="Fit to the panel" onclick={fit} disabled={zoom === 1}>Fit</button>
  </div>
  <div
    class="frame"
    class:movable={zoom > 1}
    bind:this={frame}
    onpointerdown={onDown}
    role="img"
    aria-label={label}
  >
    <div class="sheet" style:transform="translate({x}px, {y}px) scale({zoom})">
      {@html svg}
    </div>
  </div>
</div>

<style>
  .wrap { position: relative; }
  .tools { position: absolute; top: 6px; right: 6px; z-index: 1; display: flex; gap: 4px; }
  .tools button { padding: 1px 8px; background: rgba(255, 255, 255, 0.92); border: 1px solid var(--line); border-radius: 3px; cursor: pointer; }
  .tools button:disabled { color: var(--dim); cursor: default; }
  .frame { overflow: hidden; border: 1px solid var(--line); background: #fff; touch-action: none; }
  .frame.movable { cursor: grab; }
  .sheet { transform-origin: 0 0; }
  .sheet :global(svg) { display: block; width: 100%; height: auto; }
</style>

<script lang="ts">
  // One hull view. The SVG is made in Rust (the drawing crate, to the Drawing Format Standard); this
  // component only places it, fitted to the space it has, and lets the user zoom and move it. It holds
  // no geometry and no scale. When the space changes size, the view returns to the fitted extent.
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

  /** Ctrl (or Cmd) and the wheel, or a pinch, zooms. */
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

  // The window, a panel or the tab layout changed the frame: fit again.
  $effect(() => {
    const watch = new ResizeObserver(() => fit());
    watch.observe(frame);
    return () => watch.disconnect();
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
  <div class="mat">
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
  <div class="state" data-testid="zoom-state">{zoom === 1 ? "fitted" : "zoomed"}</div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; gap: var(--sp-1); }
  /* The mat: the drawing sheet sits on --surface-2, in a 1 px --rule-strong border, square corners. */
  .mat { position: relative; flex: 1; min-height: 0; box-sizing: border-box; padding: var(--sp-2); background: var(--surface-2); border: var(--lw-thin) solid var(--rule-strong); border-radius: 0; }
  .tools { position: absolute; top: var(--sp-3); right: var(--sp-3); z-index: 1; display: flex; gap: var(--sp-1); }
  .tools button { padding: 1px var(--sp-2); cursor: pointer; }
  .tools button:disabled { cursor: default; }
  /* The sheet is white in both themes: it is paper, and the standard draws black on white. */
  .frame { height: 100%; box-sizing: border-box; overflow: hidden; background: var(--sheet); touch-action: none; }
  .frame.movable { cursor: grab; }
  .sheet { transform-origin: 0 0; width: 100%; height: 100%; }
  /* The drawing has a view box, so it scales to fit both width and height. */
  .sheet :global(svg) { display: block; width: 100%; height: 100%; }
  .state { flex: none; font: var(--fs-11) / var(--lh-ui) var(--font-ui); color: var(--ink-3); }
</style>

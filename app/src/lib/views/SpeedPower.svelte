<script lang="ts">
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { onMount } from "svelte";

  // One speed-power chart for one channel environment: power and fuel against speed. It draws the
  // `curves` entry exactly as the engine gave it. The limiting speed is a dashed line; points where
  // squat limits the speed are red dots. Hover shows exact values in the legend. Zoom: drag a box,
  // or wheel; pan: shift + wheel, or drag with Alt held; double-click or the button resets.
  interface Point {
    kn: number;
    kw: number;
    fuel_lph: number;
    squat_m: number;
    squat_limited: boolean;
  }
  let { name, limitingSpeed, blockage, points }: {
    name: string;
    limitingSpeed: number | null;
    blockage: number;
    points: Point[];
  } = $props();

  let host: HTMLDivElement;
  let plot: uPlot | undefined;
  let zoomed = $state(false);

  const NAVY = "#14325a";
  const GOLD = "#b8932f";
  const RED = "#a02a1a";

  let squatCount = $derived(points.filter((p) => p.squat_limited).length);
  const exact = (_u: uPlot, v: number | null) => (v === null ? "--" : String(v));

  function limitLine(u: uPlot) {
    if (limitingSpeed === null) return;
    const x = u.valToPos(limitingSpeed, "x", true);
    const { left, top, width, height } = u.bbox;
    if (x < left || x > left + width) return;
    const ctx = u.ctx;
    ctx.save();
    ctx.strokeStyle = RED;
    ctx.fillStyle = RED;
    ctx.lineWidth = 1.5 * devicePixelRatio;
    ctx.setLineDash([6 * devicePixelRatio, 4 * devicePixelRatio]);
    ctx.beginPath();
    ctx.moveTo(x, top);
    ctx.lineTo(x, top + height);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.font = `${11 * devicePixelRatio}px sans-serif`;
    ctx.textAlign = x > left + width / 2 ? "right" : "left";
    ctx.fillText(`limit ${limitingSpeed.toFixed(2)} kn`, x + (x > left + width / 2 ? -4 : 4) * devicePixelRatio, top + 12 * devicePixelRatio);
    ctx.restore();
  }

  function options(width: number): uPlot.Options {
    return {
      width,
      height: 260,
      scales: { x: { time: false } },
      cursor: { drag: { x: true, y: false, uni: 20 } },
      hooks: {
        draw: [limitLine],
        setScale: [(u, key) => { if (key === "x") zoomed = u.scales.x.min !== points[0]?.kn || u.scales.x.max !== points[points.length - 1]?.kn; }],
      },
      axes: [
        { label: "Speed (kn)", stroke: "#444" },
        { label: "Power (kW)", stroke: NAVY, scale: "kw" },
        { label: "Fuel (l/h)", stroke: GOLD, scale: "fuel", side: 1, grid: { show: false } },
      ],
      series: [
        { label: "Speed (kn)", value: exact },
        { label: "Power (kW)", scale: "kw", stroke: NAVY, width: 2, value: exact },
        { label: "Fuel (l/h)", scale: "fuel", stroke: GOLD, width: 2, value: exact },
        {
          label: "Squat-limited (kW)",
          scale: "kw",
          paths: () => null,
          points: { show: true, size: 9, fill: RED, stroke: RED },
          value: exact,
        },
      ],
    };
  }

  function data(): uPlot.AlignedData {
    return [
      points.map((p) => p.kn),
      points.map((p) => p.kw),
      points.map((p) => p.fuel_lph),
      points.map((p) => (p.squat_limited ? p.kw : null)),
    ];
  }

  export function resetZoom() {
    plot?.setData(data(), true);
  }

  onMount(() => {
    plot = new uPlot(options(host.clientWidth || 600), data(), host);
    const over = plot.over;
    const zoomX = (factor: number, centre: number) => {
      const u = plot!;
      const { min, max } = u.scales.x;
      if (min === undefined || max === undefined) return;
      const at = u.posToVal(centre, "x");
      u.setScale("x", { min: at - (at - min) * factor, max: at + (max - at) * factor });
    };
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const u = plot!;
      const { min, max } = u.scales.x;
      if (min === undefined || max === undefined) return;
      if (e.shiftKey) {
        const shift = ((max - min) * Math.sign(e.deltaY || e.deltaX)) * 0.1;
        u.setScale("x", { min: min + shift, max: max + shift });
      } else {
        zoomX(e.deltaY < 0 ? 0.8 : 1.25, e.offsetX);
      }
    };
    let panning: { x: number; min: number; max: number } | null = null;
    const down = (e: MouseEvent) => {
      if (!e.altKey) return;
      const { min, max } = plot!.scales.x;
      panning = { x: e.clientX, min: min!, max: max! };
      e.stopPropagation();
    };
    const move = (e: MouseEvent) => {
      if (!panning) return;
      const u = plot!;
      const perPx = (panning.max - panning.min) / u.bbox.width * devicePixelRatio;
      const d = (e.clientX - panning.x) * perPx;
      u.setScale("x", { min: panning.min - d, max: panning.max - d });
    };
    const up = () => (panning = null);
    over.addEventListener("wheel", onWheel, { passive: false });
    over.addEventListener("mousedown", down, true);
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
    const ro = new ResizeObserver(() => plot?.setSize({ width: host.clientWidth, height: 260 }));
    ro.observe(host);
    return () => {
      ro.disconnect();
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
      plot?.destroy();
    };
  });

  // New results for the same chart: replace the data and keep the zoom out of the way.
  $effect(() => {
    points;
    limitingSpeed;
    plot?.setData(data(), true);
  });
</script>

<div class="chart" data-testid="speed-power-chart">
  <div class="head">
    <strong>{name}</strong>
    <span class="dim" title={`limiting speed ${limitingSpeed ?? "none"}, blockage ${blockage}`}>
      {#if limitingSpeed !== null}limiting speed {limitingSpeed.toFixed(2)} kn{:else}no speed limit from the channel{/if}
      · blockage {blockage.toFixed(3)}
      {#if squatCount > 0}· {squatCount} squat-limited point{squatCount === 1 ? "" : "s"}{/if}
    </span>
    <button onclick={resetZoom} disabled={!zoomed}>Reset zoom</button>
  </div>
  <div bind:this={host} class="host"></div>
</div>

<style>
  .chart { margin: 8px 0 18px; }
  .head { display: flex; gap: 12px; align-items: baseline; flex-wrap: wrap; font-size: 13px; }
  .dim { color: var(--dim); font-size: 12px; }
  button { margin-left: auto; font-size: 12px; }
  .host { width: 100%; }
</style>

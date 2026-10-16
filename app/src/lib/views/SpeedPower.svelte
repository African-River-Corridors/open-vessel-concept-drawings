<script lang="ts">
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { onMount } from "svelte";

  // One speed-power chart for one channel environment: power and fuel against speed. It draws the
  // `curves` entry exactly as the engine gave it. The limiting speed is a dashed line; points where
  // squat limits the speed are red dots. There is no zoom, pan or hover. The user clicks a point to
  // select it, and the readout shows that point's values. The design speed is selected to start with
  // (the point at the Option's `max_speed_kn`, or the nearest computed point).
  interface Point {
    kn: number;
    kw: number;
    fuel_lph: number;
    squat_m: number;
    squat_limited: boolean;
  }
  let { name, limitingSpeed, blockage, points, designSpeed, option }: {
    name: string;
    limitingSpeed: number | null;
    blockage: number;
    points: Point[];
    /** The Option's `max_speed_kn` input, from the Design session. */
    designSpeed: number | null;
    /** The active Option: a new Option starts again from the design speed. */
    option: string;
  } = $props();

  let host: HTMLDivElement;
  let plot: uPlot | undefined;
  /** Index in `points` of the selected point. */
  let selected = $state(0);

  const NAVY = "#14325a";
  const GOLD = "#b8932f";
  const RED = "#a02a1a";

  let squatCount = $derived(points.filter((p) => p.squat_limited).length);

  /** The point at the design speed, else the nearest one; on a tie, the lower speed. */
  function defaultIndex(list: Point[], speed: number | null): number {
    if (speed === null || list.length === 0) return 0;
    let best = 0;
    for (let i = 1; i < list.length; i++) {
      const d = Math.abs(list[i].kn - speed);
      const b = Math.abs(list[best].kn - speed);
      if (d < b || (d === b && list[i].kn < list[best].kn)) best = i;
    }
    return best;
  }

  /** The point nearest to a speed along the x axis. */
  function nearestIndex(speed: number): number {
    let best = 0;
    for (let i = 1; i < points.length; i++) {
      if (Math.abs(points[i].kn - speed) < Math.abs(points[best].kn - speed)) best = i;
    }
    return best;
  }

  const exact = (_u: uPlot, v: number | null) => (v === null ? "--" : String(v));
  const num = (v: number, dp: number) => v.toLocaleString("en-GB", { minimumFractionDigits: dp, maximumFractionDigits: dp });
  let point = $derived<Point | undefined>(points[selected]);

  /** A ring on each curve at the selected point, and a thin line down to the speed axis. */
  function marker(u: uPlot) {
    const p = points[selected];
    if (!p) return;
    const x = u.valToPos(p.kn, "x", true);
    const { left, top, width, height } = u.bbox;
    if (x < left || x > left + width) return;
    const ctx = u.ctx;
    const k = devicePixelRatio;
    ctx.save();
    ctx.strokeStyle = "#666";
    ctx.lineWidth = k;
    ctx.beginPath();
    ctx.moveTo(x, top);
    ctx.lineTo(x, top + height);
    ctx.stroke();
    for (const [scale, value, colour] of [["kw", p.kw, NAVY], ["fuel", p.fuel_lph, GOLD]] as const) {
      ctx.beginPath();
      ctx.arc(x, u.valToPos(value, scale, true), 6 * k, 0, 2 * Math.PI);
      ctx.fillStyle = "#fff";
      ctx.fill();
      ctx.lineWidth = 2.5 * k;
      ctx.strokeStyle = colour;
      ctx.stroke();
    }
    ctx.restore();
  }

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

  function options(width: number, height: number): uPlot.Options {
    return {
      width,
      height,
      scales: { x: { time: false } },
      // No hover readout, no selection box: the chart answers clicks only.
      cursor: { show: false, drag: { x: false, y: false } },
      legend: { show: false },
      hooks: {
        draw: [limitLine, marker],
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

  const hostHeight = () => Math.max(host.clientHeight, 120);

  onMount(() => {
    plot = new uPlot(options(host.clientWidth || 600, hostHeight()), data(), host);
    const onClick = (e: MouseEvent) => {
      const rect = plot!.over.getBoundingClientRect();
      selected = nearestIndex(plot!.posToVal(e.clientX - rect.left, "x"));
    };
    plot.over.addEventListener("click", onClick);
    const ro = new ResizeObserver(() => plot?.setSize({ width: host.clientWidth, height: hostHeight() }));
    ro.observe(host);
    return () => {
      ro.disconnect();
      plot?.destroy();
    };
  });

  // New results, a new design speed or another Option: replace the data and select the design speed.
  $effect(() => {
    option;
    selected = defaultIndex(points, designSpeed);
    limitingSpeed;
    plot?.setData(data(), true);
  });

  // A click changes the selected point: draw the marker again.
  $effect(() => {
    selected;
    plot?.redraw(false);
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
  </div>
  {#if point}
    <p class="readout" data-testid="speed-power-readout" data-kn={point.kn} data-design={selected === defaultIndex(points, designSpeed)}>
      <span class="sel">Selected point</span>
      <span>speed <strong>{num(point.kn, 2)} kn</strong></span>
      <span>power <strong>{num(point.kw, 0)} kW</strong></span>
      <span>fuel <strong>{num(point.fuel_lph, 1)} l/h</strong></span>
      <span>squat <strong>{num(point.squat_m, 2)} m</strong></span>
      <span>squat-limited <strong>{point.squat_limited ? "yes" : "no"}</strong></span>
      {#if limitingSpeed !== null}<span>limiting speed <strong>{num(limitingSpeed, 2)} kn</strong></span>{/if}
    </p>
  {/if}
  <div bind:this={host} class="host"></div>
</div>

<style>
  .chart { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .head { display: flex; gap: 12px; align-items: baseline; flex-wrap: wrap; font-size: 13px; }
  .dim { color: var(--dim); font-size: 12px; }
  .readout { display: flex; gap: 14px; flex-wrap: wrap; margin: 2px 0 4px; font-size: 12px; }
  .readout .sel { color: var(--dim); }
  .host { flex: 1; min-height: 0; width: 100%; }
  .host :global(.u-over) { cursor: pointer; }
</style>

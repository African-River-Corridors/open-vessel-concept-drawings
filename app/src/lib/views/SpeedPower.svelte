<script lang="ts">
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { onMount } from "svelte";
  import { axisInkHooks, currentTokens, dashFor, uiFont, watchTheme, type ChartTokens } from "../chartTheme";
  import { num, t } from "../i18n.svelte";

  // One speed-power chart for one channel environment: power and fuel against speed. It draws the
  // `curves` entry exactly as the engine gave it. The limiting speed is a dashed line; points where
  // squat limits the speed are red dots. There is no zoom, pan or hover. The user clicks a point to
  // select it, and the readout shows that point's values. The design speed is selected to start with
  // (the point at the Option's `max_speed_kn`, or the nearest computed point). Colours come from the
  // CSS tokens each time the chart draws (chartTheme.ts); a theme change redraws it, keeping the state.
  // Power is a solid line in --ink and fuel a dashed line in --accent, each named on its line, so the two
  // are never told apart by colour alone.
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

  /** The tokens now in force; read again at the start of every draw. */
  let tok: ChartTokens = currentTokens();
  const FUEL_DASH = dashFor(1);
  const scaled = (dash: number[]) => dash.map((d) => d * devicePixelRatio);

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
  let point = $derived<Point | undefined>(points[selected]);

  /** A hollow ring on each curve at the selected point, and a thin line down to the speed axis. */
  function marker(u: uPlot) {
    const p = points[selected];
    if (!p) return;
    const x = u.valToPos(p.kn, "x", true);
    const { left, top, width, height } = u.bbox;
    if (x < left || x > left + width) return;
    const ctx = u.ctx;
    const k = devicePixelRatio;
    ctx.save();
    ctx.strokeStyle = tok.ink3;
    ctx.lineWidth = k;
    ctx.beginPath();
    ctx.moveTo(x, top);
    ctx.lineTo(x, top + height);
    ctx.stroke();
    // A hollow circle on each curve: the curve shows through it.
    for (const [scale, value, colour] of [["kw", p.kw, tok.ink], ["fuel", p.fuel_lph, tok.accent]] as const) {
      ctx.beginPath();
      ctx.arc(x, u.valToPos(value, scale, true), 6 * k, 0, 2 * Math.PI);
      ctx.lineWidth = 2 * k;
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
    ctx.strokeStyle = tok.fail;
    ctx.fillStyle = tok.fail;
    ctx.lineWidth = 1.5 * devicePixelRatio;
    ctx.setLineDash([6 * devicePixelRatio, 4 * devicePixelRatio]);
    ctx.beginPath();
    ctx.moveTo(x, top);
    ctx.lineTo(x, top + height);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.font = uiFont(tok, 11 * devicePixelRatio);
    ctx.textAlign = x > left + width / 2 ? "right" : "left";
    ctx.fillText(t("app-sp-limit-line", { speed: num(limitingSpeed, 2) }), x + (x > left + width / 2 ? -4 : 4) * devicePixelRatio, top + 12 * devicePixelRatio);
    ctx.restore();
  }

  /** Squat-limited points: a cross (a shape, not a colour), drawn over the power curve. */
  function squatMarks(u: uPlot) {
    const k = devicePixelRatio;
    const { left, top, width, height } = u.bbox;
    const ctx = u.ctx;
    ctx.save();
    ctx.strokeStyle = tok.fail;
    ctx.lineWidth = 2 * k;
    for (const p of points) {
      if (!p.squat_limited) continue;
      const x = u.valToPos(p.kn, "x", true);
      const y = u.valToPos(p.kw, "kw", true);
      if (x < left || x > left + width || y < top || y > top + height) continue;
      ctx.beginPath();
      ctx.moveTo(x - 4 * k, y - 4 * k);
      ctx.lineTo(x + 4 * k, y + 4 * k);
      ctx.moveTo(x - 4 * k, y + 4 * k);
      ctx.lineTo(x + 4 * k, y - 4 * k);
      ctx.stroke();
    }
    ctx.restore();
  }

  /** Each curve is named on its line, above it: power three quarters along, fuel under half way. */
  function curveNames(u: uPlot) {
    if (points.length === 0) return;
    const k = devicePixelRatio;
    const { top, height } = u.bbox;
    const ctx = u.ctx;
    const at = (fraction: number) => points[Math.round(fraction * (points.length - 1))];
    const power = at(0.75);
    const fuel = at(0.4);
    ctx.save();
    ctx.font = uiFont(tok, 11 * k, 500);
    ctx.textAlign = "center";
    ctx.textBaseline = "bottom";
    const clamp = (y: number) => Math.min(Math.max(y, top + 14 * k), top + height - 4 * k);
    ctx.fillStyle = tok.ink;
    ctx.fillText(t("app-sp-power"), u.valToPos(power.kn, "x", true), clamp(u.valToPos(power.kw, "kw", true) - 8 * k));
    ctx.fillStyle = tok.accent;
    ctx.fillText(t("app-sp-fuel"), u.valToPos(fuel.kn, "x", true), clamp(u.valToPos(fuel.fuel_lph, "fuel", true) - 8 * k));
    ctx.restore();
  }

  function options(width: number, height: number): uPlot.Options {
    const ink = axisInkHooks(() => tok);
    const t0 = tok;
    const axis = {
      stroke: () => tok.ink3,
      font: uiFont(t0, 11),
      labelFont: uiFont(t0, 12, 600),
      grid: { stroke: () => tok.rule, width: 1 },
      ticks: { stroke: () => tok.rule, width: 1 },
      border: { show: false },
    };
    return {
      width,
      height,
      scales: { x: { time: false } },
      // No hover readout, no selection box: the chart answers clicks only.
      cursor: { show: false, drag: { x: false, y: false } },
      legend: { show: false },
      hooks: {
        // Read the tokens first, then the axes draw their text in two inks, then the marks over the curves.
        drawClear: [() => (tok = currentTokens()), ink.drawClear],
        drawAxes: [ink.drawAxes],
        draw: [limitLine, squatMarks, curveNames, marker],
      },
      axes: [
        { ...axis, label: t("app-sp-speed-axis") },
        { ...axis, label: t("app-sp-power-axis"), scale: "kw" },
        { ...axis, label: t("app-sp-fuel-axis"), scale: "fuel", side: 1, grid: { show: false } },
      ],
      series: [
        { label: t("app-sp-speed-axis"), value: exact },
        { label: t("app-sp-power-axis"), scale: "kw", stroke: () => tok.ink, width: 2, value: exact },
        { label: t("app-sp-fuel-axis"), scale: "fuel", stroke: () => tok.accent, width: 2, dash: scaled(FUEL_DASH), value: exact },
        {
          label: t("app-sp-squat-series"),
          scale: "kw",
          paths: () => null,
          points: { show: false },
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
    // A theme change: draw again in the new colours. The data, the selected point and the size stay.
    const stopTheme = watchTheme(() => plot?.redraw(false));
    // The bundled font may arrive after the first draw: draw again, with the axes measured again.
    void document.fonts?.ready.then(() => plot?.redraw(true, true));
    return () => {
      stopTheme();
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
    <span class="dim" title={t("app-sp-head-title", { speed: limitingSpeed === null ? t("app-none") : String(limitingSpeed), blockage: String(blockage) })}>
      {#if limitingSpeed !== null}{t("app-sp-limiting-speed", { speed: num(limitingSpeed, 2) })}{:else}{t("app-sp-no-limit")}{/if}
      {t("app-sp-blockage", { blockage: num(blockage, 3) })}
      {#if squatCount > 0}{t("app-sp-squat-points", { count: squatCount })}{/if}
    </span>
  </div>
  {#if point}
    <p class="readout" data-testid="speed-power-readout" data-kn={point.kn} data-design={selected === defaultIndex(points, designSpeed)}>
      <span class="sel">{t("app-sp-selected")}</span>
      <span>{t("app-sp-r-speed")} <strong>{num(point.kn, 2)} kn</strong></span>
      <span>{t("app-sp-r-power")} <strong>{num(point.kw, 0)} kW</strong></span>
      <span>{t("app-sp-r-fuel")} <strong>{num(point.fuel_lph, 1)} l/h</strong></span>
      <span>{t("app-sp-r-squat")} <strong>{num(point.squat_m, 2)} m</strong></span>
      <span>{t("app-sp-r-squat-limited")} <strong>{t(point.squat_limited ? "app-yes" : "app-no")}</strong></span>
      {#if limitingSpeed !== null}<span>{t("app-sp-r-limiting-speed")} <strong>{num(limitingSpeed, 2)} kn</strong></span>{/if}
    </p>
  {/if}
  <div bind:this={host} class="host"></div>
</div>

<style>
  .chart { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .head { display: flex; gap: 12px; align-items: baseline; flex-wrap: wrap; font-size: 13px; }
  .dim { color: var(--ink-3); font-size: 12px; }
  .readout { display: flex; gap: 14px; flex-wrap: wrap; margin: 2px 0 4px; font-size: 12px; }
  .readout .sel { color: var(--ink-3); }
  .host { flex: 1; min-height: 0; width: 100%; background: var(--surface); border: 1px solid var(--rule); box-sizing: border-box; }
  .host :global(.u-over) { cursor: pointer; }
</style>

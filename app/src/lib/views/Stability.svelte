<script lang="ts">
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { onMount, untrack } from "svelte";
  import { axisInkHooks, currentTokens, dashFor, hatchPattern, placeLabels, uiFont, watchTheme, type ChartTokens } from "../chartTheme";
  import { language, num, say, t, ticks, type TextIds } from "../i18n.svelte";

  // The stability results of a gas Cargo, exactly as the engine gives them: a GZ curve per loading
  // condition (the governing one drawn thick), the heel angles of the criteria and the angle of
  // maximum GZ marked, and below the chart the criteria set, pass or fail, the limitations and a table
  // of conditions with the criteria of each. The chart zooms and pans; Fit returns to the whole curve.
  // Numbers are rounded for reading; the full value is on hover. Colours come from the CSS tokens each
  // time the chart draws (chartTheme.ts), and a theme change redraws it with the zoom kept. The governing
  // curve is solid in --ink; the others are dashed in --accent, each with its own dash pattern and its
  // name on the line, so no curve is told apart by colour alone.
  interface Criterion {
    name: string;
    value: number;
    limit: number;
    ok: boolean;
    source: string;
  }
  interface Condition {
    name: string;
    cargo_t: number;
    fill: number;
    displacement_t: number;
    draught_m: number;
    kg_m: number;
    gm_solid_m: number;
    free_surface_m: number;
    gm_fluid_m: number;
    max_gz_m: number;
    max_gz_deg: number;
    /** `[heel in degrees, GZ in metres]`, from the engine. */
    gz: [number, number][];
    deck_edge_deg: number | null;
    criteria: Criterion[];
  }
  interface StabilityResults {
    criteria_set: string;
    ok: boolean;
    lowest_gm_condition: string;
    limitations: string[];
    conditions: Condition[];
  }
  /** What the engine says to mark on the curves (`stability_marks` of the results). */
  interface Marks {
    criteria_angles_deg: number[];
  }
  let { stability, marks, ids }: { stability: StabilityResults | null | undefined; marks: Marks | null | undefined; ids: TextIds | null } = $props();
  /** Engine text (a condition or criterion name, a limitation) in the Display Language. */
  const tr = (text: string) => say(text, ids);

  /** The tokens now in force; read again at the start of every draw. */
  let tok: ChartTokens = currentTokens();
  const governing = (c: Condition) => c.name === stability?.lowest_gm_condition;
  /** The colour of a condition's curve: the governing one is --ink, the others --accent. */
  const colour = (c: Condition) => (governing(c) ? tok.ink : tok.accent);
  /** The dash pattern of a condition's curve: solid for the governing one, then one pattern each. */
  const dash = (c: Condition, i: number) => {
    if (governing(c)) return dashFor(0);
    const before = (stability?.conditions ?? []).slice(0, i).filter((o) => !governing(o)).length;
    return dashFor(before + 1);
  };
  const scaled = (d: number[]) => d.map((v) => v * devicePixelRatio);

  let host = $state<HTMLDivElement>();
  let plot: uPlot | undefined;
  let zoomed = $state(false);
  let fitted = { x: [0, 1], y: [0, 1] };

  /** One shared heel axis, and one GZ column per condition (null where a condition has no point). */
  function data(): uPlot.AlignedData {
    const conditions = stability?.conditions ?? [];
    const angles = [...new Set(conditions.flatMap((c) => c.gz.map((p) => p[0])))].sort((a, b) => a - b);
    return [angles, ...conditions.map((c) => {
      const at = new Map(c.gz.map((p) => [p[0], p[1]]));
      return angles.map((a) => at.get(a) ?? null);
    })];
  }

  /** The band between the two criteria angles, a line at each, and a dot at each curve's maximum. */
  function marksDraw(u: uPlot) {
    const k = devicePixelRatio;
    const { left, top, width, height } = u.bbox;
    const ctx = u.ctx;
    ctx.save();
    const angles = marks?.criteria_angles_deg ?? [];
    if (angles.length === 2) {
      const [x0, x1] = angles.map((a) => u.valToPos(a, "x", true));
      // The band between the criteria angles: a light tint and hatching, so it shows without colour.
      ctx.globalAlpha = 0.12;
      ctx.fillStyle = tok.gilt;
      ctx.fillRect(x0, top, x1 - x0, height);
      ctx.globalAlpha = 0.5;
      const hatch = hatchPattern(ctx, tok.gilt, k);
      if (hatch) {
        ctx.fillStyle = hatch;
        ctx.fillRect(x0, top, x1 - x0, height);
      }
      ctx.globalAlpha = 1;
    }
    ctx.strokeStyle = tok.gilt;
    ctx.fillStyle = tok.ink2;
    ctx.lineWidth = k;
    ctx.setLineDash([5 * k, 4 * k]);
    ctx.font = uiFont(tok, 11 * k);
    for (const a of angles) {
      const x = u.valToPos(a, "x", true);
      if (x < left || x > left + width) continue;
      ctx.beginPath();
      ctx.moveTo(x, top);
      ctx.lineTo(x, top + height);
      ctx.stroke();
    }
    ctx.setLineDash([]);
    // Each angle is written on a ground of the chart surface, so that a curve under it does not cross it.
    ctx.textAlign = "left";
    ctx.textBaseline = "alphabetic";
    for (const a of angles) {
      const x = u.valToPos(a, "x", true);
      if (x < left || x > left + width) continue;
      const text = `${a}\u00b0`;
      ctx.fillStyle = tok.surface;
      ctx.fillRect(x + 1 * k, top + 1 * k, ctx.measureText(text).width + 4 * k, 14 * k);
      ctx.fillStyle = tok.ink2;
      ctx.fillText(text, x + 3 * k, top + 12 * k);
    }
    (stability?.conditions ?? []).forEach((c) => {
      const x = u.valToPos(c.max_gz_deg, "x", true);
      const y = u.valToPos(c.max_gz_m, "y", true);
      if (x < left || x > left + width || y < top || y > top + height) return;
      // A hollow circle at the maximum of the curve.
      ctx.beginPath();
      ctx.arc(x, y, 4 * k, 0, 2 * Math.PI);
      ctx.lineWidth = 2 * k;
      ctx.strokeStyle = colour(c);
      ctx.stroke();
    });
    // Each curve is named on its line, at its last point inside the plot, above the line.
    ctx.font = uiFont(tok, 11 * k, 500);
    ctx.textAlign = "right";
    ctx.textBaseline = "bottom";
    const names: { name: string; x: number; y: number; ink: string }[] = [];
    for (const c of stability?.conditions ?? []) {
      const last = [...c.gz].reverse().find(([a]) => a >= (u.scales.x.min ?? -Infinity) && a <= (u.scales.x.max ?? Infinity));
      if (!last) continue;
      const y = u.valToPos(last[1], "y", true);
      if (y < top || y > top + height) continue;
      names.push({ name: tr(c.name), x: Math.min(u.valToPos(last[0], "x", true), left + width) - 4 * k, y: y - 5 * k, ink: colour(c) });
    }
    // Names that would meet are moved apart, up or down, and kept inside the plot (issue #667). A
    // name with no room left is not written: the legend above the chart names every curve.
    const at = placeLabels(names.map((n) => n.y), top + 12 * k, top + height - 2 * k, 13 * k);
    names.forEach((n, i) => {
      const y = at[i];
      if (y === null) return;
      ctx.fillStyle = n.ink;
      ctx.fillText(n.name, n.x, y);
    });
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
      values: (_u: uPlot, ts: number[]) => ticks(ts),
    };
    return {
      width,
      height,
      scales: { x: { time: false } },
      cursor: { show: false, drag: { x: false, y: false } },
      legend: { show: false },
      hooks: {
        drawClear: [() => (tok = currentTokens()), ink.drawClear],
        drawAxes: [ink.drawAxes],
        draw: [marksDraw],
        setScale: [() => (zoomed = isZoomed())],
      },
      axes: [
        { ...axis, label: t("app-stab-heel-axis") },
        // A tick about every 28 pixels, written to two decimals, so GZ can be read off the axis.
        { ...axis, label: t("app-stab-gz-axis"), space: 28, values: (_u, ts) => ts.map((v) => num(v, 2)), size: 56 },
      ],
      series: [
        {},
        ...(stability?.conditions ?? []).map((c, i) => ({
          label: tr(c.name),
          stroke: () => colour(c),
          width: governing(c) ? 2 : 1.5,
          dash: scaled(dash(c, i)),
          points: { show: false },
        })),
      ],
    };
  }

  function isZoomed(): boolean {
    if (!plot) return false;
    const near = (a: number | null | undefined, b: number) => Math.abs((a ?? b) - b) < 1e-9;
    const { x, y } = plot.scales;
    return !(near(x.min, fitted.x[0]) && near(x.max, fitted.x[1]) && near(y.min, fitted.y[0]) && near(y.max, fitted.y[1]));
  }

  /** The extent that shows every curve whole: all heel angles, and GZ from zero (or below) to the top. */
  function extent() {
    const [angles, ...columns] = data();
    const gz = columns.flat().filter((v): v is number => typeof v === "number");
    const top = Math.max(...gz, 0.1);
    const bottom = Math.min(...gz, 0);
    const pad = (top - bottom) * 0.08;
    return { x: [angles[0], angles[angles.length - 1]], y: [bottom - (bottom < 0 ? pad : 0), top + pad] };
  }

  /** The whole curve in the panel. */
  function fit() {
    if (!plot || !host) return;
    fitted = extent();
    plot.setSize({ width: host.clientWidth, height: Math.max(host.clientHeight, 120) });
    plot.setData(data(), false);
    plot.setScale("x", { min: fitted.x[0], max: fitted.x[1] });
    plot.setScale("y", { min: fitted.y[0], max: fitted.y[1] });
    zoomed = false;
  }

  /** Zoom about a point of the plot, given as the fraction of the width and height (0 to 1). */
  function zoomAt(factor: number, fx: number, fy: number) {
    if (!plot) return;
    for (const [key, f] of [["x", fx], ["y", 1 - fy]] as const) {
      const { min, max } = plot.scales[key];
      if (min == null || max == null) continue;
      const at = min + (max - min) * f;
      plot.setScale(key, { min: at - (at - min) * factor, max: at + (max - at) * factor });
    }
  }

  onMount(() => {
    if (!host || !stability) return;
    plot = new uPlot(options(host.clientWidth || 600, Math.max(host.clientHeight, 120)), data(), host);
    fit();
    const over = plot.over;
    const onWheel = (e: WheelEvent) => {
      if (!(e.ctrlKey || e.metaKey)) return;
      e.preventDefault();
      const box = over.getBoundingClientRect();
      zoomAt(Math.exp(e.deltaY * 0.002), (e.clientX - box.left) / box.width, (e.clientY - box.top) / box.height);
    };
    over.addEventListener("wheel", onWheel, { passive: false });
    // Drag to move the zoomed curve.
    const onDown = (e: PointerEvent) => {
      if (!zoomed) return;
      over.setPointerCapture(e.pointerId);
      let last = { x: e.clientX, y: e.clientY };
      const move = (m: PointerEvent) => {
        const u = plot!;
        const box = over.getBoundingClientRect();
        for (const [key, d, size] of [["x", last.x - m.clientX, box.width], ["y", m.clientY - last.y, box.height]] as const) {
          const { min, max } = u.scales[key];
          if (min == null || max == null) continue;
          const shift = (d / size) * (max - min);
          u.setScale(key, { min: min + shift, max: max + shift });
        }
        last = { x: m.clientX, y: m.clientY };
      };
      const up = () => {
        over.removeEventListener("pointermove", move);
        over.removeEventListener("pointerup", up);
      };
      over.addEventListener("pointermove", move);
      over.addEventListener("pointerup", up);
    };
    over.addEventListener("pointerdown", onDown);
    // The window or a panel changed the space: fit again.
    const ro = new ResizeObserver(() => fit());
    ro.observe(host);
    // A theme change: draw again in the new colours. Zoom, pan and the data stay.
    const stopTheme = watchTheme(() => plot?.redraw(false));
    // The bundled font may arrive after the first draw: draw again, with the axes measured again.
    void document.fonts?.ready.then(() => plot?.redraw(true, true));
    return () => {
      stopTheme();
      ro.disconnect();
      plot?.destroy();
    };
  });

  const r = (v: number | null, dp: number) =>
    v === null || v === undefined ? "–" : num(v, dp);

  const COLUMNS: { label: string; unit: string; dp: number; get: (c: Condition) => number | null }[] = [
    { label: "app-stab-col-cargo", unit: "t", dp: 1, get: (c) => c.cargo_t },
    { label: "app-stab-col-fill", unit: "", dp: 2, get: (c) => c.fill },
    { label: "app-stab-col-displacement", unit: "t", dp: 1, get: (c) => c.displacement_t },
    { label: "app-stab-col-draught", unit: "m", dp: 2, get: (c) => c.draught_m },
    { label: "app-stab-col-kg", unit: "m", dp: 2, get: (c) => c.kg_m },
    { label: "app-stab-col-gm-solid", unit: "m", dp: 2, get: (c) => c.gm_solid_m },
    { label: "app-stab-col-free-surface", unit: "m", dp: 2, get: (c) => c.free_surface_m },
    { label: "app-stab-col-gm-fluid", unit: "m", dp: 2, get: (c) => c.gm_fluid_m },
    { label: "app-stab-col-max-gz", unit: "m", dp: 2, get: (c) => c.max_gz_m },
    { label: "app-stab-col-max-gz-angle", unit: "deg", dp: 0, get: (c) => c.max_gz_deg },
    { label: "app-stab-col-deck-edge", unit: "deg", dp: 1, get: (c) => c.deck_edge_deg },
  ];

  // Another Display Language: the axis titles again, then a full draw (tick numbers, curve names).
  // Only the language is tracked here; the data has its own effect.
  $effect(() => {
    language();
    untrack(() => {
      if (!plot) return;
      [t("app-stab-heel-axis"), t("app-stab-gz-axis")].forEach((title, i) => (plot!.axes[i].label = title));
      plot.redraw(false, true);
    });
  });
</script>

<div class="stab" data-testid="stability">
  {#if !stability}
    <div data-testid="stability-wip">
      <h3 class="wip">{t("app-work-in-development")}</h3>
      <p class="note">{t("app-stab-wip")}</p>
    </div>
  {:else}
    <div class="chartbox" data-testid="stability-chart">
      <div class="legend">
        {#each stability.conditions as c, i (c.name)}
          <span class="item" class:governing-item={c.name === stability.lowest_gm_condition} data-condition={c.name}>
            <svg class="swatch" class:gov-swatch={c.name === stability.lowest_gm_condition} width="28" height="8" viewBox="0 0 28 8" aria-hidden="true">
              <line x1="0" y1="4" x2="28" y2="4" stroke-width={c.name === stability.lowest_gm_condition ? 2 : 1.5} stroke-dasharray={dash(c, i).join(" ") || undefined} />
            </svg>
            <strong>{tr(c.name)}</strong>{#if c.name === stability.lowest_gm_condition}<span class="govtag">{t("app-stab-governs")}</span>{/if}
            {t("app-stab-legend", { gm: r(c.gm_fluid_m, 2), gz: r(c.max_gz_m, 2), angle: r(c.max_gz_deg, 0) })}
          </span>
        {/each}
        <span class="tools" role="group" aria-label={t("app-stab-zoom")}>
          <button type="button" aria-label={t("app-zoom-out")} onclick={() => zoomAt(1.5, 0.5, 0.5)}>−</button>
          <button type="button" aria-label={t("app-zoom-in")} onclick={() => zoomAt(1 / 1.5, 0.5, 0.5)}>+</button>
          <button type="button" aria-label={t("app-fit-panel")} onclick={fit} disabled={!zoomed}>{t("app-fit")}</button>
        </span>
      </div>
      <div class="host" bind:this={host} class:movable={zoomed}></div>
    </div>

    <div class="details">
      <p class="summary">
        <span class="tag {stability.ok ? 'ok' : 'fail'}">{t(stability.ok ? "app-pass" : "app-fail")}</span>
        <strong>{tr(stability.criteria_set)}</strong>
        <span class="dim">{t("app-stab-governing")} <strong>{tr(stability.lowest_gm_condition)}</strong></span>
      </p>

      <table class="conditions">
        <thead>
          <tr>
            <th>{t("app-stab-condition")}</th>
            {#each COLUMNS as col (col.label)}
              <th class="num">{t(col.label)}{#if col.unit}<span class="unit">&nbsp;({col.unit})</span>{/if}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each stability.conditions as c (c.name)}
            <tr class:governing={c.name === stability.lowest_gm_condition} data-governing={c.name === stability.lowest_gm_condition}>
              <th>
                {tr(c.name)}
                {#if c.name === stability.lowest_gm_condition}<span class="gov">{t("app-stab-governs")}</span>{/if}
              </th>
              {#each COLUMNS as col (col.label)}
                {@const v = col.get(c)}
                <td class="num" title={v === null ? t("app-not-applicable") : String(v)}>{r(v, col.dp)}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>

      {#each stability.conditions as c (c.name)}
        <h4>{t("app-stab-criteria-of", { condition: tr(c.name) })}</h4>
        <table class="criteria">
          <tbody>
            {#each c.criteria as k (k.name)}
              <tr class:failed={!k.ok}>
                <td><span class="tag {k.ok ? 'ok' : 'fail'}">{t(k.ok ? "app-pass" : "app-fail")}</span></td>
                <th>{tr(k.name)}</th>
                <td class="num" title={String(k.value)}>{r(k.value, 3)}</td>
                <td class="dim">{t("app-stab-needs-at-least")}</td>
                <td class="num" title={String(k.limit)}>{r(k.limit, 3)}</td>
                <td class="dim">{k.source}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/each}

      <!-- The engine's own limitations, then the known ones of the method that no result carries
           (issue #282: the areas ignore the down-flooding angle). -->
      <h4>{t("app-stab-limitations")}</h4>
      <ul>
        {#each stability.limitations as l}<li>{tr(l)}</li>{/each}
        <li>{t("app-stab-limit-downflooding")}</li>
      </ul>
    </div>
  {/if}
</div>

<style>
  .stab { height: 100%; display: flex; flex-direction: column; gap: 8px; }
  .chartbox { flex: 3; min-height: 210px; display: flex; flex-direction: column; }
  .legend { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 16px; font-size: 12px; margin-bottom: 4px; }
  .item.governing-item { font-weight: 600; }
  /* A sample of the curve's line, with its dash pattern. */
  .swatch { margin-right: 4px; vertical-align: middle; }
  .swatch line { stroke: var(--accent); }
  .swatch.gov-swatch line { stroke: var(--ink); }
  .govtag { font-size: 11px; background: var(--accent); color: var(--accent-ink); padding: 0 5px; border-radius: var(--radius); margin: 0 4px; text-transform: uppercase; }
  .tools { margin-left: auto; display: flex; gap: 4px; }
  .tools button { padding: 1px 8px; background: var(--surface); color: var(--ink); border: 1px solid var(--rule-strong); border-radius: var(--radius); cursor: pointer; }
  .tools button:disabled { color: var(--ink-3); cursor: default; }
  .host { flex: 1; min-height: 0; width: 100%; background: var(--surface); border: 1px solid var(--rule); box-sizing: border-box; }
  .host.movable :global(.u-over) { cursor: grab; }
  /* The tables keep their size, so this part has its own scroll area; the panel itself does not scroll. */
  .details { flex: 2; min-height: 0; overflow: auto; }
  .summary { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; margin: 0 0 8px; }
  .dim { color: var(--ink-3); font-size: 12px; }
  .note { color: var(--ink-3); }
  .wip { font-family: var(--font-serif); color: var(--ink); font-size: 16px; margin: 0 0 4px; }
  table { border-collapse: collapse; font-size: 12px; }
  .conditions { display: block; overflow-x: auto; max-width: 100%; }
  th { text-align: left; font-weight: normal; padding: 3px 10px 3px 0; white-space: nowrap; }
  thead th { color: var(--ink-3); vertical-align: bottom; }
  td { padding: 3px 8px; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  th.num { text-align: right; }
  .unit { color: var(--ink-3); }
  tr.governing { background: var(--accent-soft); }
  tr.governing th { font-weight: 600; }
  .gov { font-size: 11px; background: var(--accent); color: var(--accent-ink); padding: 0 5px; border-radius: var(--radius); margin-left: 4px; text-transform: uppercase; }
  h4 { font-family: var(--font-serif); font-size: 13px; margin: 14px 0 4px; color: var(--ink); }
  .tag { font-size: 11px; padding: 0 5px; border-radius: var(--radius); }
  .tag.ok { background: var(--pass-soft); color: var(--pass); }
  .tag.ok::before { content: "\2713\00a0"; }
  .tag.fail { background: var(--fail-soft); color: var(--fail); }
  .tag.fail::before { content: "\2715\00a0"; }
  tr.failed th { color: var(--fail); }
  ul { margin: 4px 0; padding-left: 18px; font-size: 12px; color: var(--ink-3); }
</style>

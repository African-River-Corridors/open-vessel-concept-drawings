<script lang="ts">
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { onMount } from "svelte";

  // The stability results of a gas Cargo, exactly as the engine gives them: a GZ curve per loading
  // condition (the governing one drawn thick), the heel angles of the criteria and the angle of
  // maximum GZ marked, and below the chart the criteria set, pass or fail, the limitations and a table
  // of conditions with the criteria of each. The chart zooms and pans; Fit returns to the whole curve.
  // Numbers are rounded for reading; the full value is on hover.
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
    governing: string;
    limitations: string[];
    conditions: Condition[];
  }
  /** What the engine says to mark on the curves (`stability_marks` of the results). */
  interface Marks {
    criteria_angles_deg: number[];
  }
  let { stability, marks }: { stability: StabilityResults | null | undefined; marks: Marks | null | undefined } = $props();

  const NAVY = "#14325a";
  const OTHERS = ["#2f7d6d", "#b8932f", "#7a5ea8", "#8a8f98"];
  /** The colour of a condition's curve: the governing one is navy and thick. */
  const colour = (c: Condition, i: number) => (c.name === stability?.governing ? NAVY : OTHERS[i % OTHERS.length]);

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
      ctx.fillStyle = "rgba(184, 147, 47, 0.12)";
      ctx.fillRect(x0, top, x1 - x0, height);
    }
    ctx.strokeStyle = "#8a7a4a";
    ctx.fillStyle = "#6b5a2a";
    ctx.lineWidth = k;
    ctx.setLineDash([5 * k, 4 * k]);
    ctx.font = `${11 * k}px sans-serif`;
    for (const a of angles) {
      const x = u.valToPos(a, "x", true);
      if (x < left || x > left + width) continue;
      ctx.beginPath();
      ctx.moveTo(x, top);
      ctx.lineTo(x, top + height);
      ctx.stroke();
      ctx.fillText(`${a}\u00b0`, x + 3 * k, top + 12 * k);
    }
    ctx.setLineDash([]);
    (stability?.conditions ?? []).forEach((c, i) => {
      const x = u.valToPos(c.max_gz_deg, "x", true);
      const y = u.valToPos(c.max_gz_m, "y", true);
      if (x < left || x > left + width || y < top || y > top + height) return;
      ctx.beginPath();
      ctx.arc(x, y, 4 * k, 0, 2 * Math.PI);
      ctx.fillStyle = "#fff";
      ctx.fill();
      ctx.lineWidth = 2 * k;
      ctx.strokeStyle = colour(c, i);
      ctx.stroke();
    });
    ctx.restore();
  }

  function options(width: number, height: number): uPlot.Options {
    return {
      width,
      height,
      scales: { x: { time: false } },
      cursor: { show: false, drag: { x: false, y: false } },
      legend: { show: false },
      hooks: {
        draw: [marksDraw],
        setScale: [() => (zoomed = isZoomed())],
      },
      axes: [
        { label: "Heel angle (deg)", stroke: "#444" },
        // A tick about every 28 pixels, written to two decimals, so GZ can be read off the axis.
        { label: "GZ (m)", stroke: "#444", space: 28, values: (_u, ticks) => ticks.map((v) => v.toFixed(2)), size: 56 },
      ],
      series: [
        {},
        ...(stability?.conditions ?? []).map((c, i) => ({
          label: c.name,
          stroke: colour(c, i),
          width: c.name === stability?.governing ? 3 : 1.5,
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
    return () => {
      ro.disconnect();
      plot?.destroy();
    };
  });

  const r = (v: number | null, dp: number) =>
    v === null || v === undefined ? "–" : v.toLocaleString("en-GB", { minimumFractionDigits: dp, maximumFractionDigits: dp });

  const COLUMNS: { label: string; unit: string; dp: number; get: (c: Condition) => number | null }[] = [
    { label: "Cargo", unit: "t", dp: 1, get: (c) => c.cargo_t },
    { label: "Fill", unit: "", dp: 2, get: (c) => c.fill },
    { label: "Displacement", unit: "t", dp: 1, get: (c) => c.displacement_t },
    { label: "Draught", unit: "m", dp: 2, get: (c) => c.draught_m },
    { label: "KG", unit: "m", dp: 2, get: (c) => c.kg_m },
    { label: "GM solid", unit: "m", dp: 2, get: (c) => c.gm_solid_m },
    { label: "Free surface", unit: "m", dp: 2, get: (c) => c.free_surface_m },
    { label: "GM fluid", unit: "m", dp: 2, get: (c) => c.gm_fluid_m },
    { label: "Max GZ", unit: "m", dp: 2, get: (c) => c.max_gz_m },
    { label: "Angle of max GZ", unit: "deg", dp: 0, get: (c) => c.max_gz_deg },
    { label: "Deck-edge angle", unit: "deg", dp: 1, get: (c) => c.deck_edge_deg },
  ];
</script>

<div class="stab" data-testid="stability">
  {#if !stability}
    <div data-testid="stability-wip">
      <h3 class="wip">Work in development</h3>
      <p class="note">Stability is computed for gas cargo only. Dry-bulk stability is work in development.</p>
    </div>
  {:else}
    <div class="chartbox" data-testid="stability-chart">
      <div class="legend">
        {#each stability.conditions as c, i (c.name)}
          <span class="item" class:governing-item={c.name === stability.governing} data-condition={c.name}>
            <span class="swatch" style:background={colour(c, i)}></span>
            <strong>{c.name}</strong>{#if c.name === stability.governing}<span class="govtag">governs</span>{/if}
            GM {r(c.gm_fluid_m, 2)} m · max GZ {r(c.max_gz_m, 2)} m at {r(c.max_gz_deg, 0)}°
          </span>
        {/each}
        <span class="tools" role="group" aria-label="Zoom the GZ curves">
          <button type="button" aria-label="Zoom out" onclick={() => zoomAt(1.5, 0.5, 0.5)}>−</button>
          <button type="button" aria-label="Zoom in" onclick={() => zoomAt(1 / 1.5, 0.5, 0.5)}>+</button>
          <button type="button" aria-label="Fit to the panel" onclick={fit} disabled={!zoomed}>Fit</button>
        </span>
      </div>
      <div class="host" bind:this={host} class:movable={zoomed}></div>
    </div>

    <div class="details">
      <p class="summary">
        <span class="tag {stability.ok ? 'ok' : 'fail'}">{stability.ok ? "PASS" : "FAIL"}</span>
        <strong>{stability.criteria_set}</strong>
        <span class="dim">Governing condition: <strong>{stability.governing}</strong></span>
      </p>

      <table class="conditions">
        <thead>
          <tr>
            <th>Condition</th>
            {#each COLUMNS as col (col.label)}
              <th class="num">{col.label}{#if col.unit}<span class="unit">&nbsp;({col.unit})</span>{/if}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each stability.conditions as c (c.name)}
            <tr class:governing={c.name === stability.governing} data-governing={c.name === stability.governing}>
              <th>
                {c.name}
                {#if c.name === stability.governing}<span class="gov">governs</span>{/if}
              </th>
              {#each COLUMNS as col (col.label)}
                {@const v = col.get(c)}
                <td class="num" title={v === null ? "not applicable" : String(v)}>{r(v, col.dp)}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>

      {#each stability.conditions as c (c.name)}
        <h4>{c.name}: criteria</h4>
        <table class="criteria">
          <tbody>
            {#each c.criteria as k (k.name)}
              <tr class:failed={!k.ok}>
                <td><span class="tag {k.ok ? 'ok' : 'fail'}">{k.ok ? "PASS" : "FAIL"}</span></td>
                <th>{k.name}</th>
                <td class="num" title={String(k.value)}>{r(k.value, 3)}</td>
                <td class="dim">needs at least</td>
                <td class="num" title={String(k.limit)}>{r(k.limit, 3)}</td>
                <td class="dim">{k.source}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/each}

      {#if stability.limitations.length > 0}
        <h4>Limitations</h4>
        <ul>
          {#each stability.limitations as l}<li>{l}</li>{/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

<style>
  .stab { height: 100%; display: flex; flex-direction: column; gap: 8px; }
  .chartbox { flex: 3; min-height: 210px; display: flex; flex-direction: column; }
  .legend { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 16px; font-size: 12px; margin-bottom: 4px; }
  .item.governing-item { font-weight: 600; }
  .swatch { display: inline-block; width: 14px; height: 4px; margin-right: 4px; vertical-align: middle; }
  .govtag { font-size: 10px; background: var(--gold); color: #fff; padding: 0 5px; border-radius: 3px; margin: 0 4px; text-transform: uppercase; }
  .tools { margin-left: auto; display: flex; gap: 4px; }
  .tools button { padding: 1px 8px; background: #fff; border: 1px solid var(--line); border-radius: 3px; cursor: pointer; }
  .tools button:disabled { color: var(--dim); cursor: default; }
  .host { flex: 1; min-height: 0; width: 100%; border: 1px solid var(--line); box-sizing: border-box; }
  .host.movable :global(.u-over) { cursor: grab; }
  /* The tables keep their size, so this part has its own scroll area; the panel itself does not scroll. */
  .details { flex: 2; min-height: 0; overflow: auto; }
  .summary { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; margin: 0 0 8px; }
  .dim { color: var(--dim); font-size: 12px; }
  .note { color: var(--dim); }
  .wip { font-family: var(--serif); color: var(--navy); font-size: 16px; margin: 0 0 4px; }
  table { border-collapse: collapse; font-size: 12px; }
  .conditions { display: block; overflow-x: auto; max-width: 100%; }
  th { text-align: left; font-weight: normal; padding: 3px 10px 3px 0; white-space: nowrap; }
  thead th { color: var(--dim); vertical-align: bottom; }
  td { padding: 3px 8px; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  th.num { text-align: right; }
  .unit { color: var(--dim); }
  tr.governing { background: #fdf1d3; }
  tr.governing th { font-weight: 600; }
  .gov { font-size: 10px; background: var(--gold); color: #fff; padding: 0 5px; border-radius: 3px; margin-left: 4px; text-transform: uppercase; }
  h4 { font-family: var(--serif); font-size: 13px; margin: 14px 0 4px; color: var(--navy); }
  .tag { font-size: 10px; padding: 0 5px; border-radius: 3px; }
  .tag.ok { background: #e1f0e1; color: #245c24; }
  .tag.fail { background: #fbe4e1; color: #a02a1a; }
  tr.failed th { color: #7a1d12; }
  ul { margin: 4px 0; padding-left: 18px; font-size: 12px; color: var(--dim); }
</style>

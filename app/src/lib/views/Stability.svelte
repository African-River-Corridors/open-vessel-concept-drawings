<script lang="ts">
  // The stability results of a gas Cargo, exactly as the engine gives them: the criteria set, pass
  // or fail overall, the governing condition, the limitations, and a table of conditions with the
  // criteria of each. Numbers are rounded for reading; the full value is on hover.
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
  let { stability }: { stability: StabilityResults | null | undefined } = $props();

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

<div data-testid="stability">
  {#if !stability}
    <p class="note" data-testid="stability-not-computed">Stability is not computed for this Cargo.</p>
  {:else}
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
  {/if}
</div>

<style>
  .summary { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; margin: 0 0 8px; }
  .dim { color: var(--dim); font-size: 12px; }
  .note { color: var(--dim); }
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
